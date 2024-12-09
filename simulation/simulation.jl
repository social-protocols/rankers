# agent
# -- - get ranking page
# -- - post / comment
# -- - vote
# ---
# voting
# -- - rank effect
# -- - page effect

using HTTP
using Dates
using JSON
using Random
using Distributions

host_url = "http://localhost:3000"
endpoints = Dict(
    :items => "/items",
    :vote_events => "/vote_events",
    :rankings_hn => "/rankings/hn",
    :rankings_qn => "/rankings/qn",
    :rankings_newest => "/rankings/newest",
)

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


function now_utc_millis()
    return Int(floor(datetime2unix(Dates.now(UTC)) * 1000))
end


function get_ranking(host::String, endpoint::String)
    response = HTTP.get(host * endpoint)
    response_json = JSON.parse(String(response.body))
    return response_json
end


function vote_prob_at_rank(rank::Int, total_items::Int)
    @assert(rank <= total_items, "Rank must be smaller than total_items")
    normalization_constant = sum([1 / k for k = 1:total_items])
    return (1 / normalization_constant) * (1 / rank)
end


function send_item!(
    item_buf::Array{Int},
    author_id::String,
    host::String,
    endpoint::String,
    comment_probability::Float64,
)
    (item_id, parent_id) =
        isempty(item_buf) ? (1, nothing) : (maximum(item_buf) + 1, rand(item_buf))

    item = Dict(
        "item_id" => item_id,
        "parent_id" => parent_id,
        "author_id" => author_id,
        "created_at" => now_utc_millis(),
    )
    headers = Dict("Content-Type" => "application/json")

    println("Creating item: ", item_id, "...")

    HTTP.post(host_url * endpoint, headers, body = JSON.json(item))

    push!(item_buf, item_id)
end

function send_vote_event!(
    vote_event_buf::Array{Int},
    item_buf::Array{Int},
    user_id::String,
    host::String,
    endpoint::String,
)
    vote_event_id = isempty(vote_event_buf) ? 1 : maximum(vote_event_buf) + 1
    item_id = rand(item_buf)

    vote_event = Dict(
        "vote_event_id" => vote_event_id,
        "item_id" => item_id,
        "user_id" => user_id,
        "vote" => 1,
        "created_at" => now_utc_millis(),
    )
    headers = Dict("Content-Type" => "application/json")

    println("Creating vote event: ", vote_event_id, "...")

    HTTP.post(host_url * endpoint, headers, body = JSON.json(vote_event))

    push!(vote_event_buf, vote_event_id)
end



# ------------------------------------------------------


# Insert seed items before simulation starts

comment_probability = 0.8
item_to_vote_ratio = 0.05

ranking_page_distribution = Dict(
    1 => RankingPage(HN, 0.45),
    2 => RankingPage(QN, 0.45),
    3 => RankingPage(Newest, 0.1),
)

item_buffer = Int[]
vote_event_buffer = Int[]


users = string.(collect(1:100))

for i = 1:1000
    if (rand() > item_to_vote_ratio) && !isempty(item_buffer)
        send_vote_event!(
            vote_event_buffer,
            item_buffer,
            rand(users),
            host_url,
            endpoints[:vote_events],
        )
    else
        send_item!(
            item_buffer,
            rand(users),
            host_url,
            endpoints[:items],
            comment_probability,
        )
    end
    sleep(0.1)
end
