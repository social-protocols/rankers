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

include("model.jl")
include("client.jl")
include("util.jl")

HOST_URL = "http://localhost:3000"
ENDPOINTS = Dict(
    :items => "/items",
    :vote_events => "/vote_events",
    :rankings_hn => "/rankings/hn",
    :rankings_qn => "/rankings/qn",
    :rankings_newest => "/rankings/newest",
)
RANKING_PAGE_DISTRIBUTION = Dict(
    1 => RankingPage(HN, 0.45),
    2 => RankingPage(QN, 0.45),
    3 => RankingPage(Newest, 0.1),
)

model = setup_model(HOST_URL, ENDPOINTS, 3, RANKING_PAGE_DISTRIBUTION, 100, 10)

run!(model, 5)
