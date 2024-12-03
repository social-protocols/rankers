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

post_id_buffer = Int[]

function create_post!(buf::Array{Int}, host::String, endpoint::String, comment_probability::Float64)
    post_id = isempty(buf) ? 1 : maximum(buf) + 1
    parent_id = rand() < comment_probability && isempty(buf) ? nothing : rand(buf)

    post = Dict(
        "post_id" => post_id,
        "parent_id" => parent_id,
        "content" => "Halo i bims, lol",
        "created_at" => Int(floor(Dates.time())),
    )
    headers = Dict("Content-Type" => "application/json")

    println("Attempting to create post", post)

    HTTP.post(
        host_url * endpoint,
        headers,
        body=JSON.json(post)
    )

    push!(buf, post_id)
end

for i in 1:100
    create_post!(
        post_id_buffer,
        host_url,
        endpoints[:create_post],
        comment_probability,
    )
    sleep(1)
end
