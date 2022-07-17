math.randomseed(os.time())

-- this takes a single goal move and creates a goal that will repeat that goal move `reps` times
local function repeat_goal_move(goal_move, reps)
    local new_goal_data = {
        goal_move = goal_move,
        reps = 0,
        max_reps = reps,
        is_satisfied = function(self)
            return self.reps <= self.max_reps
        end,
        next_step = function(self)
            if self:is_satisfied() then
                return self.goal_move
            end
        end,
        made_move = function(self, history)
            local agrees
            if self.goal_move.pursuer == nil then
                agrees = true
            else 
                agrees = self.goal_move.pursuer == history.speaker
            end
            local satisfied = history:was_move_satisfied(self.goal_move.dialog_move)
            if agrees and satisfied then
                self.reps = self.reps + 1
            end
        end,
    }
    return {
        next_step = function(_)
            return new_goal_data:next_step()
        end,
        made_move = function(_, history)
            return new_goal_data:made_move(history)
        end,
        is_satisfied = function()
            return new_goal_data:is_satisfied()
        end,
    }
end

local dialog_manager = DialogManager(
    {
        -- nothing node
        nothing = {
            dialog_moves = {},
            addressed_topics = {},
            precondition = nil,
            edit_historical_move = nil,
            formatter = function(...)
                return ""
            end,
            parts = {},
        },
        simple_hello_node = {
            dialog_moves = {},
            addressed_topics = {},
            precondition = nil,
            edit_historical_move = nil,
            formatter = function(...)
                return "Hello"
            end,
            parts = {},
        },
        simple_hi_node = {
            dialog_moves = {},
            addressed_topics = {},
            precondition = nil,
            edit_historical_move = nil,
            formatter = function(...)
                return "Hi"
            end,
            parts = {},
        },
        simple_greet_node = {
            dialog_moves = {"greet"},
            addressed_topics = {},
            precondition = nil,
            edit_historical_move = nil,
            formatter = function(_, parts)
                return table.concat(parts) .. "."
            end,
            parts = {{"simple_hello_node", "simple_hi_node"}},
        },
        raw_fav_music_genre = {
            dialog_moves = {"raw_fav_music_genre"},
            addressed_topics = {},
            precondition = nil,
            edit_historical_move = function(_, hmove)
                hmove:get_my_obligations():address("state feelings")
            end,
            formatter = function(state, parts)
                return state:get_my_state().character.truths["favorite music genre"].text
            end,
            parts = {},
        },
        state_fav_music_genre = {
            dialog_moves = {"state fav music genre", "make small talk"},
            addressed_topics = {"fav music genre"},
            precondition = function(state)
                local obligated = state
                    :get_my_state()
                    :get_obligation("state fav music genre")
                    ~= nil
                return obligated or state.topic_state:can_be_addressed("fav music genre")
            end,
            edit_historical_move = function(state, hmove)
                hmove:get_my_obligations():address("state fav music genre")
                hmove.topic_state:address("fav music genre")
                local speaker = hmove.speaker
                local source = state:get_speaker_state(speaker).character
                local my_fav_music_genre = source.truths["favorite music genre"]
                -- placeholder: we don't actually use locations in this
                local location = source
                local other = state:get_speaker_state(not speaker).character
                local other_of_source = other.models[source.name]
                other_of_source["favorite music genre"][my_fav_music_genre]:insert{
                    data = {},
                    kind = {
                        kind = "statement",
                        source = source,
                        location = location,
                    },
                    strength = 100,
                }
                other_of_source:recompute_total_strengths()
                other_of_source:recompute_strongest()
            end,
            formatter = function(state, parts)
                return "My favorite genre of music is " .. table.concat(parts) .. "."
            end,
            parts = {{"raw_fav_music_genre"}},
        },
        ask_fav_music_genre = {
            dialog_moves = {"ask fav music genre", "make small talk"},
            addressed_topics = {},
            precondition = function(state)
                return state.topic_state:can_be_introduced("fav music genre")
            end,
            edit_historical_move = function(state, hmove)
                hmove:get_others_obligations():push(
                    "state fav music genre",
                    0,
                    3
                );
                hmove.topic_state:introduce("fav music genre");
            end,
            formatter = function(...)
                return "What's your favorite genre of music?"
            end,
            parts = {},
        },
        state_feelings = {
            dialog_moves = {"state feelings", "make small talk"},
            addressed_topics = {},
            precondition = nil,
            edit_historical_move = function(_, hmove)
                hmove:get_my_obligations():address("state feelings")
            end,
            formatter = function(...)
                local words = {
                    "happy",
                    "sad",
                    "angry",
                    "upset",
                    "ecstatic",
                    "excited",
                }
                local feeling = words[math.random(#words)]
                return "I feel " .. feeling .. "."
            end,
            parts = {},
        },
        ask_feelings = {
            dialog_moves = {"ask feelings", "make small talk"},
            addressed_topics = {},
            precondition = nil,
            edit_historical_move = function(_, hmove)
                hmove:get_others_obligations():push("state feelings", 0, 3)
            end,
            formatter = function(...)
                return "How are you feeling?"
            end,
            parts = {},
        },
    },
    {
        function(state)
            state.person0:insert_obligation("greet", {
                urgency = 1000000,
                time_to_live = 5,
                times_pushed = 0,
            })
            state.person1:insert_obligation("greet", {
                urgency = 1000000,
                time_to_live = 5,
                times_pushed = 0,
            })
            state:insert_goal(repeat_goal_move({
                pursuer = nil,
                dialog_move = "make small talk",
            }, 2))
        end,
    }
)

table.contains = function(table, element)
    for k,v in pairs(table) do
        if element == v then
            return k
        end
    end
end

local entities = {}
local facets = {}

local function make_simple_facet(facet, values)
    local function make_options(value)
        local new_values = {}
        for _, new_value in ipairs(values) do
            if new_value ~= value then
                table.insert(new_values, value)
            end
        end
        return new_values
    end

    facets[facet] = {
        facet = facet,
        values = {},
    }

    for _, value in ipairs(values) do
        facets[facet].values[value] = {
            facet = facet,
            text = value,
            hash_string = facet .. " " .. value,
            try_mutate = function(_, _, evidence_id)
                local options = make_options(value)
                return options[math.random(#options)]
            end,
        }
    end
end

make_simple_facet("favorite music genre", {
    "jazz",
    "rock",
    "metal",
    "calypso",
})

local function make_simple_entity(name, facet_values)
    entities[name] = {
        name = name,
        facets = {},
        truths = {},
        models = {},
        relevant_facets = function(self)
            return self.facets
        end,
        facet_truth = function(self, facet)
            return self.truths[facet]
        end,
        is_facet_relevant = function(self, facet)
            return table.contains(self.facets, facet)
        end,
    }
    local entity = entities[name]
    for facet, true_value in pairs(facet_values) do
        table.insert(entity.facets, facet)
        entity.truths[facet] = facets[facet].values[true_value]
    end
    return entity
end

local alice = make_simple_entity("Alice", {
    ["favorite music genre"] = "jazz",
})
local bob = make_simple_entity("Bob", {
    ["favorite music genre"] = "metal"
})
alice.models[alice.name] = ReflexiveModel(alice)
bob.models[bob.name] = ReflexiveModel(bob)
local alice_of_bob = EvidenceModel(alice, bob)
alice.models[bob.name] = alice_of_bob
local bob_of_alice = EvidenceModel(bob, alice)
bob.models[alice.name] = bob_of_alice

local conversation = dialog_manager:new_conversation(
    false,
    0.2,
    alice,
    bob
)

while not conversation.done do
    dialog_manager:step_conversation(conversation, "make small talk")
end

for _, hmove in conversation:history_iter() do
    local name
    if hmove.speaker then
        name = "Alice"
    else
        name = "Bob"
    end
    name = name .. ":"
    print(name, hmove.words)
end

local function print_strongest_beliefs(model, facets)
    local holder_name = model.holder.name
    local regarding_name = model.regarding.name .. "'s"
    for facet, data in model:iter_facets() do
        local strongest = data.strongest
        if strongest then
            local text = holder_name .. " believes " .. regarding_name .. " " .. facet .. " is "
                .. strongest.text .. "."
            print(text)
        else
            local text = holder_name .. " doesn't know about " .. regarding_name .. " " .. facet
                .. "."
            print(text)
        end
    end
end

print_strongest_beliefs(alice_of_bob, {"favorite music genre"})
print_strongest_beliefs(bob_of_alice, {"favorite music genre"})
