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
    Newest => "/rankings/newest",
    HackerNews => "/rankings/hn",
    QualityNews => "/rankings/qn",
)
RANKING_PAGE_DISTRIBUTION = Dict(
    1 => RankingPage(HackerNews, 0.45),
    2 => RankingPage(QualityNews, 0.45),
    3 => RankingPage(Newest, 0.1),
)

model = setup_model(HOST_URL, ENDPOINTS, 3, RANKING_PAGE_DISTRIBUTION, 100, 10)

run!(model, 1)
