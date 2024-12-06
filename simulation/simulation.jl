using HTTP
using Dates
using JSON
using Random

host_url = "http://localhost:3000"
endpoints = Dict(
    :create_item => "/create_item",
    :send_vote_event => "/send_vote_event",
)

comment_probability = 0.8
item_to_vote_ratio = 0.05

item_buffer = Int[]
vote_event_buffer = Int[]

function now_utc_millis()
    return Int(floor(datetime2unix(Dates.now(UTC)) * 1000))
end

function create_item!(item_buf::Array{Int}, host::String, endpoint::String, comment_probability::Float64)
    (item_id, parent_id) = isempty(item_buf) ? (1, nothing) : (maximum(item_buf) + 1, rand(item_buf))

    item = Dict(
        "item_id" => item_id,
        "parent_id" => parent_id,
        "created_at" => now_utc_millis(),
    )
    headers = Dict("Content-Type" => "application/json")

    println("Creating item: ", item_id, "...")

    HTTP.post(
        host_url * endpoint,
        headers,
        body=JSON.json(item)
    )

    push!(item_buf, item_id)
end

function send_vote_event!(vote_event_buf::Array{Int}, item_buf::Array{Int}, host::String, endpoint::String)
    vote_event_id = isempty(vote_event_buf) ? 1 : maximum(vote_event_buf) + 1
    item_id = rand(item_buf)

    vote_event = Dict(
        "vote_event_id" => vote_event_id,
        "item_id" => item_id,
        "vote" => 1,
        "created_at" => now_utc_millis(),
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
    if (rand() > item_to_vote_ratio) && !isempty(item_buffer)
        send_vote_event!(
            vote_event_buffer,
            item_buffer,
            host_url,
            endpoints[:send_vote_event],
        )
    else
        create_item!(
            item_buffer,
            host_url,
            endpoints[:create_item],
            comment_probability,
        )
    end
    sleep(1)
end
