os.pullEvent = os.pullEventRaw
-- Disable termination, this is done first to prevent tiny moments of time where the turtle can be stopped

JSON = require("JSON")

Fuel = turtle.getItemDetail(16).name
Instructions = {}
Pos = {
    ["x"] = 0,
    ["y"] = 0,
    ["z"] = 0
}
Rotation = 0 -- 0 = N, 1 = E, 2 = S, 3 = W

function GetInstructions()
    local request = http.get("https://turtle.awesome.tf/instructions", {["turtle_id"] = computer.getLabel()})
    if request then
        Instructions = JSON:decode(request.readAll())
        request.close()
    else
        print("Failed to get instructions from server.")
        return false
    end
    if Instructions == nil or #Instructions == 0 then
        print("No instructions received.")
        return false
    end
    return true
end

function PostInventory()
    -- Build a list of {slot = n, name = "minecraft:stone", count = 64}
    local inv = {}
    for slot = 1, 16 do
        local detail = turtle.getItemDetail(slot)
        if detail then
            table.insert(inv, {
                slot  = slot,
                name  = detail.name,  -- e.g. "minecraft:cobblestone"
                count = detail.count  -- stack size in this slot
            })
        end
    end

    local payload = JSON:encode({
        turtle_id = computer.getLabel(),
        inventory = inv
    })

    local headers = {
        ["Content-Type"] = "application/json",
        ["turtle_id"]    = computer.getLabel(),
        ["authorization"] = "iamarealturtleyesiam"
    }

    local response = http.post("https://turtle.awesome.tf/inventory", payload, headers)

    if response then
        response.close()
        return true
    else
        print("Failed to post inventory to server.")
        return false
    end
end


function PostInfo(i)
    -- Make sure our position table (Pos) is up‑to‑date
    -- UpdatePos()

    local function inspect(fn)
        local ok, data = fn()
        if ok and data then
            return data.name -- e.g. "minecraft:dirt" 
        else
            return "minecraft:air"
        end
    end

    -- Collect visible blocks
    local visible = {
        { direction = "front", name = inspect(turtle.inspect)    },
        { direction = "up",    name = inspect(turtle.inspectUp)  },
        { direction = "down",  name = inspect(turtle.inspectDown)}
    }

    -- Assemble everything the backend cares about
    local body = JSON:encode({
        turtle_id = computer.getLabel(),
        position  = Pos,               -- {x = …, y = …, z = …}
        rotation  = Rotation,          -- 0 = N, 1 = E, 2 = S, 3 = W
        fuel      = turtle.getFuelLevel(),
        blocks    = visible,
        instruction_index = i or 0,
    })

    local headers = {
        ["Content-Type"] = "application/json",
        ["turtle_id"]    = computer.getLabel()
    }

    local res = http.post("https://turtle.awesome.tf/info", body, headers)

    if res then
        res.close()
        return true
    else
        print("Failed to post turtle info to server.")
        return false
    end
end


function RunInstructions()
    for i, instruction in ipairs(Instructions) do
        if instruction == "moveup" then
            MoveUp()
        elseif instruction == "movedown" then
            MoveDown()
        elseif instruction == "moveforward" then
            MoveForward()
        elseif instruction == "moveback" then
            MoveBack()
        elseif instruction == "turnleft" then
            Turn(-1)
        elseif instruction == "turnright" then
            Turn(1)
        elseif instruction == "dig" then
            Dig()
        elseif instruction == "digup" then
            DigUp()
        elseif instruction == "digdown" then
            DigDown()
        elseif instruction == "reportarea" then
            PostInfo()
            Turn(1)
            PostInfo()
            Turn(1)
            PostInfo()
            Turn(1)
            PostInfo()
            Turn(1)
        else
            print("Unknown instruction: " .. instruction)
        end
        PostInfo(i) -- Post info after each instruction
    end
end

function Dig()
    while turtle.dig() do end
end

function DigUp()
    while turtle.digUp() do end
end

function DigDown()
    while turtle.digDown() do end
end

function UpdatePos()
    local x, y, z = gps.locate()
    if x then
        Pos["x"] = x
        Pos["y"] = y
        Pos["z"] = z
        return
    else
        local response = http.get("https://turtle.awesome.tf/gps", {["turtle_id"] = computer.getLabel()})
        if response then
            local data = JSON:decode(response.readAll())
            if data and data.success then
                Pos["x"] = data.position.x
                Pos["y"] = data.position.y
                Pos["z"] = data.position.z
            end
        else
            -- TODO: maybe complain to master server that I am lost
            -- might not need to because if the backup GPS fails, the server will know
        end
    end
end

function DetermineOrientation()
    local Oldpos = table.shallow_copy(Pos)
    if turtle.forward() then
        UpdatePos()
        if Pos['x'] > Oldpos['x'] then
            Rotation = 1 -- East (towards positive X)
        elseif Pos['x'] < Oldpos['x'] then
            Rotation = 3 -- West (towards negative X)
        elseif Pos['z'] > Oldpos['z'] then
            Rotation = 2 -- South (towards positive Z)
        elseif Pos['z'] < Oldpos['z'] then
            Rotation = 0 -- North (towards negative Z)
        end
        turtle.back() -- Move back to original position
    else
        -- If we can't move forward, we can't determine orientation
        print("Hey idiot! Either my front is blocked or I have no fuel!")
        return false
    end

    return true
end

function Refuel()
    local fueled = false
    if turtle.getFuelLevel() < 500 then
        for i = 1,16 do
            local fuel_found = false
            if not(turtle.getItemDetail(i) == nil) then
                if turtle.getItemDetail(i).name == Fuel then
                    fuel_found = true
                end
            end
            if fuel_found then
                turtle.select(i)
                while turtle.getItemCount() > 0 do
                    turtle.refuel(1)
                    if turtle.getFuelLevel() >= 500 then
                        fueled = true
                        turtle.select(1)
                        break
                    end
                end
            end
            if fueled then break end
        end
        if not fueled then
            print("Hey idiot! I need more fuel!")
        end
    end
    return true
end

function table.shallow_copy(t)
    local t2 = {}
    for k,v in pairs(t) do
        t2[k] = v
    end
    return t2
end

function MoveForward() -- moves bot forward 1 block and updates position
    Refuel()
    if not turtle.forward() then
        Instructions = {}
        PostInfo()
        GetInstructions()
        return
    end
    if Rotation == 0 then Pos["z"]=Pos["z"]-1 end
    if Rotation == 1 then Pos["x"]=Pos["x"]+1 end
    if Rotation == 2 then Pos["z"]=Pos["z"]+1 end
    if Rotation == 3 then Pos["x"]=Pos["x"]-1 end
end

function MoveBack() -- moves bot back 1 block and updates position
    Refuel()
    if not turtle.back() then
        Instructions = {}
        PostInfo()
        GetInstructions()
        return
    end
    if Rotation == 0 then Pos["z"]=Pos["z"]+1 end
    if Rotation == 1 then Pos["x"]=Pos["x"]-1 end
    if Rotation == 2 then Pos["z"]=Pos["z"]-1 end
    if Rotation == 3 then Pos["x"]=Pos["x"]+1 end
end

function MoveUp() -- moves bot up 1 block and updates position
    Refuel()
    if not turtle.up() then
        Instructions = {}
        PostInfo()
        GetInstructions()
        return
    end
    Pos["y"] = Pos["y"] + 1
end

function MoveDown() -- moves bot down 1 block and updates position
    Refuel()
    if not turtle.down() then
        Instructions = {}
        PostInfo()
        GetInstructions()
        return
    end
    Pos["y"] = Pos["y"] - 1
end

function Turn(num) -- turns bot either left (-1) or right (+1) depending on input and updates Rotation value
    if num == 1 then 
        turtle.turnRight()
        Rotation = (Rotation+1)%4
    elseif num == -1 then
        turtle.turnLeft()
        Rotation = (Rotation-1)%4
    end
end

local function main()
    Refuel()
    UpdatePos()
    DetermineOrientation()
    print("Current orientation: " .. Rotation)
end

local function passwordListener()
    while true do
        term.clearLine()
        print("Enter password to stop: ")

        local input = read("*")       -- masked input

        if input == "a" then
            term.clear()
            term.setCursorPos(1, 1)
            print("Password accepted - stopping program…")
            sleep(2)
            return
        end

        term.clearLine()
        print("Wrong password.")
        sleep(1.5)
    end
end

-- production mode
parallel.waitForAny(main, passwordListener)

-- testing mode
-- main()