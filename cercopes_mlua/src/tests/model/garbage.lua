math.randomseed(0)

table.contains = function(table, element)
    for k,v in pairs(table) do
        if element == v then
            return k
        end
    end
end

local entities = {}

local facets = {
    ["hair color"] = {
        facet = "hair color",
        values = {},
    },
}

facets["hair color"].values.red = {
    facet = facets["hair color"].facet,
    hash_string = facets["hair color"].facet .. " red",
    try_mutate = function(model_id, evidence_id)
        local options = {
            facets["hair color"].values.green,
            facets["hair color"].values.blue,
        }
        return options[math.random(#options)]
    end,
}

facets["hair color"].values.green = {
    facet = facets["hair color"].facet,
    hash_string = facets["hair color"].facet .. " green",
    try_mutate = function(model_id, evidence_id)
        local options = {
            facets["hair color"].values.red,
            facets["hair color"].values.blue,
        }
        return options[math.random(#options)]
    end,
}

facets["hair color"].values.blue = {
    facet = facets["hair color"].facet,
    hash_string = facets["hair color"].facet .. " blue",
    try_mutate = function(model_id, evidence_id)
        local options = {
            facets["hair color"].values.red,
            facets["hair color"].values.green,
        }
        return options[math.random(#options)]
    end,
}

entities.alice = {
    facets = {
        facets["hair color"].facet,
    },
    truths = {
        [facets["hair color"].facet] = facets["hair color"].values.blue,
    },
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

entities.bob = {
    facets = {
        facets["hair color"].facet,
    },
    truths = {
        [facets["hair color"].facet] = facets["hair color"].values.green,
    },
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

local alice_of_bob = EvidenceModel(entities.alice, entities.bob)
table.insert(entities.alice.models, alice_of_bob)