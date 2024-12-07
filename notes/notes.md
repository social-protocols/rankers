# Notes

### Problem

We want to make the scheme flexible enough so that users of the service can implement different ranking pages.
However, we also need to know the rank history for each page for the quality news ranking algorithm.

**Idea:** Send metadata with vote event -> page and rank where vote occurred -> then create a Poisson model for upvote share on ranks on each page -> recalculate the model every so often (maybe hourly in the beginning, then daily, then weekly, and perhaps at some point only monthly, if the vote streams are very consistent)

With that idea, we wouldn't have to persist rank history at all in the service database.
We could get than information from the vote event log where the rank and page where the vote occurred is stored.
