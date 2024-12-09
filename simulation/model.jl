@enum RankingPageType HN QN Newest

struct RankingPage
    type::RankingPageType
    traffic_share::Float64
end

struct Agent
    user_id::String
end

Base.@kwdef struct Model
    items::Int[]
    vote_events::Int[]
    host_url::String
    api::Dict{Symbol,String}
    max_votes_per_agent::Int
    ranking_page_distribution::Dict{Int,RankingPage}
    agents::Agent[]
end

function setup_model(
    host_url::String,
    api::Dict{Symbol,String},
    max_votes_per_agent::Int,
    ranking_page_distribution::Dict{Int,RankingPage},
    n_agents::Int,
)
    return Model(
        items = Int[],
        vote_events = Int[],
        host_url = host_url,
        api = api,
        max_votes_per_agent = max_votes_per_agent,
        ranking_page_distribution = ranking_page_distribution,
        agents = [Agent(i) for i in collect(string.(1:n_agents))],
    )
end

function step!(model::Model)
    for a in model.agents
        # choose and request page to look at
        looking_at_page = choose_ranking_page(model)
        endpoint = if looking_at_page == 1
            endpoints[:rankings_hn]
        elseif looking_at_page == 2
            endpoints[:rankings_qn]
        else
            endpoints[:rankings_newest]
        end
        received_ranking = get_ranking(model.host_url, endpoint)

        # cast votes
        n_votes_cast = 0
        for r in received_ranking
            if n_votes_cast >= model.max_votes_per_agent
                break
            end
            if rand() < vote_prob_at_rank(r["rank"], length(received_ranks))
                send_vote_event!(
                    model.vote_events,
                    model.items,
                    a.user_id,
                    model.host,
                    endpoint,
                )
                n_votes_cast += 1
            end
        end

        # comment on stories

        # create stories
    end
end

function run!(model::Model, steps::Int)
    model = setup_model(TODO)
    for i = 1:steps
        step!(model)
    end
end

function choose_ranking_page(model::Model)
    dist = model.ranking_page_distribution
    page_keys = sort(collect(keys(dist)))
    dist = Categorical([model.dist[key].traffic_share for key in page_keys])
    sample = rand(dist)
    return model.ranking_page_distribution[sample]
end
