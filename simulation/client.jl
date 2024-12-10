using HTTP
using JSON
using Logging

function send_item!(model::Model, author_id::String, parent_id::Union{Int,Nothing})
    item_id = isempty(model.items) ? 1 : maximum(model.items) + 1

    item = Dict(
        "item_id" => item_id,
        "parent_id" => parent_id,
        "author_id" => author_id,
        "created_at" => now_utc_millis(),
    )
    headers = Dict("Content-Type" => "application/json")

    @info "Creating item: $item_id ..."

    response =
        HTTP.post(model.host_url * model.api[:items], headers, body = JSON.json(item))

    push!(model.items, item_id)
end

function send_vote_event!(model::Model, user_id::String)
    vote_event_id = isempty(model.vote_events) ? 1 : maximum(model.vote_events) + 1
    item_id = rand(model.items)

    vote_event = Dict(
        "vote_event_id" => vote_event_id,
        "item_id" => item_id,
        "user_id" => user_id,
        "vote" => 1,
        "created_at" => now_utc_millis(),
    )
    headers = Dict("Content-Type" => "application/json")

    @info "Creating vote event: $vote_event_id ..."

    response = HTTP.post(
        model.host_url * model.api[:vote_events],
        headers,
        body = JSON.json(vote_event),
    )

    push!(model.vote_events, vote_event_id)
end

function get_ranking(model::Model, endpoint::String)
    response = HTTP.get(model.host_url * endpoint)
    response_json = JSON.parse(String(response.body))
    return response_json
end
