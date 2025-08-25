os.pullEvent = os.pullEventRaw
-- This is gonna be password protected as well
local authenticated = false
local password = "CHANGE_ME"
    while true do
        if not authenticated then
            term.clearLine()
            print("Enter password to access controller: ")
            local input = read()
            if input == password then
                authenticated = true
                term.clear()
            else
                print("Incorrect password.")
            end
        else
            term.clear()
            print("Welcome to the controller!")
            print("DO NOT FORGET TO TYPE 'logout' BEFORE LEAVING!")
            print("Type 'help' for a list of commands.")
            local input = read()
            if input == "logout" then
                authenticated = false
                term.clear()
            elseif input == "help" then
                print("Available commands:")
                print("- logout: Log out of the controller")
                print("- help: Show this help message")
                print("- exit: Exit the controller")
            elseif input == "exit" then
                print("Exiting controller...")
                break
            else
                print("Unknown command: " .. input)
            end
        end
    end