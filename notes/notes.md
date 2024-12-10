# Notes

### Problem: Adding More Ranking Pages

We want to make the scheme flexible enough so that users of the service can implement different ranking pages.
However, we also need to know the rank history for each page for the quality news ranking algorithm.

**Idea:** Send metadata with vote event -> page and rank where vote occurred -> then create a Poisson model for upvote share on ranks on each page -> recalculate the model every so often (maybe hourly in the beginning, then daily, then weekly, and perhaps at some point only monthly, if the vote streams are very consistent)

With that idea, we wouldn't have to persist rank history at all in the service database.
We could get than information from the vote event log where the rank and page where the vote occurred is stored.

### Problem: Estimate Expected Upvotes

**Idea**

rank sampling:

- in each sampling interval (let's say every five minutes), track the rank history (e.g., every minute) -> create a rank profile for that interval

upvote share prediction:

- predict upvote share from rank profile
- seeding: to enable quality news ranking, we need some time to gather data -> so at first, just set QN rank = HN rank -> do that for a while, then train the model to predict upvote share -> then the feedback loop works 
