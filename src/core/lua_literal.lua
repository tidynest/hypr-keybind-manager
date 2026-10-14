-- Renders a Lua value back as source text, for dispatcher arguments.
--
-- Loaded by lua_config.rs. Used by the prelude to record the arguments of
-- parsed binds, and on its own to turn arguments typed in the editor into
-- plain data before they are written. In strict mode a value that has no
-- literal form, such as a function or nil, raises an error; otherwise it is
-- written as `nil`.

local KEYWORDS = {}
for word in ([[and break do else elseif end false for function goto if in
               local nil not or repeat return then true until while]]):gmatch("%a+") do
    KEYWORDS[word] = true
end

local function literal(value, depth, strict)
    local kind = type(value)
    -- %q writes a newline as a backslash and a line break; keep binds on one line
    if kind == "string" then return (string.format("%q", value):gsub("\\\n", "\\n")) end
    if kind == "boolean" then return tostring(value) end
    if kind == "number" then
        -- tostring gives `inf` and `nan`, which would read back as globals
        local finite = value == value and value ~= math.huge and value ~= -math.huge
        return finite and tostring(value) or string.format("%q", value)
    end
    if kind ~= "table" or depth > 8 then
        if strict then error("arguments can only hold strings, numbers, booleans and tables", 0) end
        return "nil"
    end

    local parts, n = {}, #value
    for i = 1, n do parts[#parts + 1] = literal(value[i], depth + 1, strict) end
    local keys = {}
    for k in pairs(value) do
        if not (math.type(k) == "integer" and k >= 1 and k <= n) then keys[#keys + 1] = k end
    end
    table.sort(keys, function(a, b) return tostring(a) < tostring(b) end)
    for _, k in ipairs(keys) do
        local name = (type(k) == "string" and k:match("^[%a_][%w_]*$") and not KEYWORDS[k]) and k
            or ("[" .. literal(k, depth + 1, strict) .. "]")
        parts[#parts + 1] = name .. " = " .. literal(value[k], depth + 1, strict)
    end
    if #parts == 0 then return "{}" end
    return "{ " .. table.concat(parts, ", ") .. " }"
end

return literal
