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

include("client.jl")
include("util.jl")
include("model.jl")

host_url = "http://localhost:3000"
endpoints = Dict(
    :items => "/items",
    :vote_events => "/vote_events",
    :rankings_hn => "/rankings/hn",
    :rankings_qn => "/rankings/qn",
    :rankings_newest => "/rankings/newest",
)

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
