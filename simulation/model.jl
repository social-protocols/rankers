@enum RankingPageType HN QN Newest

struct RankingPage
    type::RankingPageType
    traffic_share::Float64
end

struct Agent
    user_id::String
end

Base.@kwdef struct Model
    items::Array{Int}
    vote_events::Array{Int}
    host_url::String
    api::Dict{Symbol,String}
    max_votes_per_agent::Int
    ranking_page_distribution::Dict{Int,RankingPage}
    agents::Array{Agent}
end

function setup_model(
    host_url::String,
    api::Dict{Symbol,String},
    max_votes_per_agent::Int,
    ranking_page_distribution::Dict{Int,RankingPage},
    n_agents::Int,
    n_seed_posts::Int,
)
    model = Model(
        items = Int[],
        vote_events = Int[],
        host_url = host_url,
        api = api,
        max_votes_per_agent = max_votes_per_agent,
        ranking_page_distribution = ranking_page_distribution,
        agents = [Agent(i) for i in collect(string.(1:n_agents))],
    )
    seed_posts!(model, n_seed_posts)
    return model
end

function seed_posts!(model::Model, n::Int)
    seed_agents = rand(model.agents, n)
    for a in seed_agents
        send_item!(model, a.user_id, nothing)
    end
end

function step!(model::Model)
    for a in model.agents
        # choose and request page to look at
        looking_at_page = choose_ranking_page(model)
        endpoint = if looking_at_page == 1
            model.api[:rankings_hn]
        elseif looking_at_page == 2
            model.api[:rankings_qn]
        else
            model.api[:rankings_newest]
        end
        received_ranking = get_ranking(model, endpoint)

        n_votes_cast = 0
        for r in received_ranking
            # cast votes
            if (n_votes_cast <= model.max_votes_per_agent) &
               (rand() < vote_prob_at_rank(r["rank"], length(received_ranking)))
                send_vote_event!(model, a.user_id)
                n_votes_cast += 1
            end

            # comment on stories
            if rand() < 0.005 # TODO
                send_item!(model, a.user_id, r["item_id"])
            end
        end

        # create stories
        for i = 1:3 # TODO
            if rand() < 0.005
                send_item!(model, a.user_id, nothing)
            end
        end

        sleep(0.3)
    end
end

function run!(model::Model, steps::Int)
    for i = 1:steps
        step!(model)
    end
end

function choose_ranking_page(model::Model)
    dist = model.ranking_page_distribution
    page_keys = sort(collect(keys(dist)))
    dist = Categorical([dist[key].traffic_share for key in page_keys])
    sample = rand(dist)
    return model.ranking_page_distribution[sample]
end
