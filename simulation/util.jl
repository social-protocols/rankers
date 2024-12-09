function now_utc_millis()
    return Int(floor(datetime2unix(Dates.now(UTC)) * 1000))
end

function vote_prob_at_rank(rank::Int, total_items::Int)
    @assert(rank <= total_items, "Rank must be smaller than total_items")
    normalization_constant = sum([1 / k for k = 1:total_items])
    return (1 / normalization_constant) * (1 / rank)
end
