# TODO:
#   - [ ] implement random voting with appropriate probability model (for now,
#         just uniform distribution over all posts)

using HTTP
using Dates
using JSON
using Random

host_url = "http://localhost:3000"
endpoints = Dict(
    :create_post => "/create_post",
    :send_vote_event => "/send_vote_event",
)

comment_probability = 0.8
post_to_vote_ratio = 0.05

post_buffer = Int[]
vote_event_buffer = Int[]

function create_post!(post_buf::Array{Int}, host::String, endpoint::String, comment_probability::Float64)
    (post_id, parent_id) = isempty(post_buf) ? (1, nothing) : (maximum(post_buf) + 1, rand(post_buf))

    post = Dict(
        "post_id" => post_id,
        "parent_id" => parent_id,
        "created_at" => Int(floor(Dates.time())),
    )
    headers = Dict("Content-Type" => "application/json")

    println("Creating post: ", post_id, "...")

    HTTP.post(
        host_url * endpoint,
        headers,
        body=JSON.json(post)
    )

    push!(post_buf, post_id)
end

function send_vote_event!(vote_event_buf::Array{Int}, post_buf::Array{Int}, host::String, endpoint::String)
    vote_event_id = isempty(vote_event_buf) ? 1 : maximum(vote_event_buf) + 1
    post_id = rand(post_buf)

    vote_event = Dict(
        "vote_event_id" => vote_event_id,
        "post_id" => post_id,
        "vote" => 1,
        "vote_event_time" => Int(floor(Dates.time())),
    )
    headers = Dict("Content-Type" => "application/json")

    println("Creating vote event: ", vote_event_id, "...")

    HTTP.post(
        host_url * endpoint,
        headers,
        body=JSON.json(vote_event)
    )

    push!(vote_event_buf, vote_event_id)
end

for i in 1:1000
    if (rand() > post_to_vote_ratio) && !isempty(post_buffer)
        send_vote_event!(
            vote_event_buffer,
            post_buffer,
            host_url,
            endpoints[:send_vote_event],
        )
    else
        create_post!(
            post_buffer,
            host_url,
            endpoints[:create_post],
            comment_probability,
        )
    end
    sleep(1)
end
