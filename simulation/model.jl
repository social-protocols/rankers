@enum RankingPageType Newest QualityNews HackerNews

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
    api::Dict{RankingPageType,String}
    max_votes_per_agent::Int
    ranking_page_distribution::Dict{Int,RankingPage}
    agents::Array{Agent}
end

function setup_model(
    host_url::String,
    api::Dict{RankingPageType,String},
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

function seed_posts!(model::Model, n::Int)::Model
    seed_agents = rand(model.agents, n)
    for a in seed_agents
        send_item!(model, a.user_id, nothing)
        sleep(0.1)
    end

    return model
end

function step!(model::Model)::Model
    for a in model.agents
        # choose and request page to look at
        ranking_page = choose_ranking_page(model)
        received_ranking = get_ranking(model, model.api[ranking_page])

        n_votes_cast = 0
        for r in received_ranking
            # cast votes
            if (n_votes_cast <= model.max_votes_per_agent) &
               (rand() < vote_prob_at_rank(r["rank"], length(received_ranking)))
                send_vote_event!(model, a.user_id, r["rank"], ranking_page)
                sleep(0.1)
                n_votes_cast += 1
            end

            # comment on stories
            if rand() < 0.005 # TODO
                send_item!(model, a.user_id, r["item_id"])
                sleep(0.1)
            end
        end

        # create stories
        for i = 1:3 # TODO
            if rand() < 0.005
                send_item!(model, a.user_id, nothing)
                sleep(0.1)
            end
        end

        sleep(0.3)
    end

    return model
end

function run!(model::Model, steps::Int)::Model
    for i = 1:steps
        step!(model)
    end

    return model
end

function choose_ranking_page(model::Model)::RankingPageType
    page_dist = model.ranking_page_distribution
    page_keys = sort(collect(keys(page_dist)))
    dist = Categorical([page_dist[key].traffic_share for key in page_keys])
    choice = rand(dist)

    return page_dist[choice].type
end
