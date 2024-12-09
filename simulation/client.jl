using HTTP
using JSON

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

function get_ranking(host::String, endpoint::String)
    response = HTTP.get(host * endpoint)
    response_json = JSON.parse(String(response.body))
    return response_json
end
