-- Recording stand-in for Hyprland's `hl` Lua API.
--
-- Loaded by lua_config.rs with two arguments: a function that reads a module
-- from the config directory, and one that reports the file and line of the
-- code calling `hl.bind`. Returns the sandbox environment the config runs in,
-- the recorded binds and the files that were required.
--
-- Every `hl.bind` call is recorded with the file and line it came from, so
-- binds created in loops are found and can be told apart from ones written
-- out by hand. Everything else in `hl` is a no-op.

local read_module, locate = ...

local records, files = {}, {}
local current_submap = nil

-- Renders a Lua value back as source text, for dispatcher arguments.
local function literal(value, depth)
    local kind = type(value)
    if kind == "string" then return string.format("%q", value) end
    if kind == "number" or kind == "boolean" then return tostring(value) end
    if kind ~= "table" or depth > 8 then return "nil" end

    local parts, n = {}, #value
    for i = 1, n do parts[#parts + 1] = literal(value[i], depth + 1) end
    local keys = {}
    for k in pairs(value) do
        if not (math.type(k) == "integer" and k >= 1 and k <= n) then keys[#keys + 1] = k end
    end
    table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
    for _, k in ipairs(keys) do
        local name = (type(k) == "string" and k:match("^[%a_][%w_]*$")) and k
            or ("[" .. literal(k, depth + 1) .. "]")
        parts[#parts + 1] = name .. " = " .. literal(value[k], depth + 1)
    end
    if #parts == 0 then return "{}" end
    return "{ " .. table.concat(parts, ", ") .. " }"
end

-- `hl.dsp.window.close()` walks a chain of nodes and ends in a descriptor.
local DISPATCHER = {}
local function dsp_node(path)
    return setmetatable({}, {
        __index = function(_, key)
            return dsp_node(path == "" and key or (path .. "." .. key))
        end,
        __call = function(_, ...)
            local args = table.pack(...)
            local rendered = {}
            for i = 1, args.n do rendered[i] = literal(args[i], 0) end
            return setmetatable({
                path = path,
                args = table.concat(rendered, ", "),
                exec = (args.n == 1 and type(args[1]) == "string") and args[1] or nil,
            }, DISPATCHER)
        end,
    })
end

-- Answers any call or index with itself, for API surface we do not model.
local stub = {}
setmetatable(stub, {
    __index = function() return stub end,
    __call = function() return stub end,
    __len = function() return 0 end,
})

local handle = {}
function handle.remove(self) self.record.removed = true end
handle.unbind = handle.remove
function handle.set_enabled() end
function handle.is_enabled() return true end

local hl = {}

function hl.bind(keys, action, opts)
    local file, line = locate()
    local record = {
        keys = tostring(keys),
        opts = type(opts) == "table" and opts or {},
        submap = current_submap,
        file = file,
        line = line,
    }
    if getmetatable(action) == DISPATCHER then
        record.kind = "dispatcher"
        record.path = action.path
        record.args = action.args
        record.exec = action.exec
    elseif type(action) == "function" then
        record.kind = "function"
    else
        record.kind = "other"
    end
    records[#records + 1] = record
    return setmetatable({ record = record }, { __index = handle })
end

function hl.unbind(keys)
    for i = #records, 1, -1 do
        local record = records[i]
        if record.keys == keys and not record.removed then
            record.removed = true
            return
        end
    end
end

function hl.define_submap(name, reset_or_fn, fn)
    local body = type(reset_or_fn) == "function" and reset_or_fn or fn
    local previous = current_submap
    current_submap = tostring(name)
    if type(body) == "function" then body() end
    current_submap = previous
end

function hl.get_current_submap() return current_submap or "" end
function hl.version() return "0.56.0" end
function hl.get_config() return nil end
function hl.is_key_down() return false end

for _, name in ipairs({
    "get_layers", "get_windows", "get_monitors", "get_workspaces",
    "get_workspace_windows", "get_loaded_plugins",
}) do
    hl[name] = function() return {} end
end
for _, name in ipairs({
    "get_active_window", "get_last_window", "get_last_workspace", "get_monitor",
    "get_monitor_at", "get_monitor_at_cursor", "get_urgent_window", "get_window",
    "get_workspace",
}) do
    hl[name] = function() return nil end
end

hl.dsp = dsp_node("")
hl.layout = stub
hl.notification = stub
hl.plugin = stub
setmetatable(hl, { __index = function() return function() return stub end end })

-- The sandbox: no io, no load, no os.execute, require limited to the config directory.
local env = {
    hl = hl,
    assert = assert, error = error, ipairs = ipairs, pairs = pairs, next = next,
    select = select, tostring = tostring, tonumber = tonumber, type = type,
    pcall = pcall, xpcall = xpcall, rawget = rawget, rawset = rawset,
    rawequal = rawequal, rawlen = rawlen, setmetatable = setmetatable,
    getmetatable = getmetatable, unpack = table.unpack,
    string = string, table = table, math = math, utf8 = utf8, _VERSION = _VERSION,
    os = { getenv = os.getenv, date = os.date, time = os.time, clock = os.clock, difftime = os.difftime },
    print = function() end,
}
env._G = env

local loaded = {}
function env.require(name)
    name = tostring(name)
    if loaded[name] ~= nil then return loaded[name] end
    local source, path = read_module(name)
    if not source then
        error("module '" .. name .. "' not found in the config directory", 2)
    end
    files[#files + 1] = path
    local chunk, err = load(source, "@" .. path, "t", env)
    if not chunk then error(err, 2) end
    local result = chunk(name)
    if result == nil then result = true end
    loaded[name] = result
    return result
end

return { env = env, records = records, files = files }
