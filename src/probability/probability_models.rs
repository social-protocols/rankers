use probability::prelude::*;

#[allow(dead_code)]
#[derive(Debug)]
struct Tally {
    upvotes: i32,
    total_votes: i32,
}

#[allow(dead_code)]
trait Update<T> {
    type Output: Update<T>;
    fn update(&self, new_data: &T) -> Self::Output;
}

impl Update<Tally> for Beta {
    type Output = Beta;
    fn update(&self, new_data: &Tally) -> Beta {
        let new_upvotes = new_data.upvotes as f64;
        let new_downvotes = (new_data.total_votes - new_data.upvotes) as f64;
        Beta::new(self.alpha() + new_upvotes, self.beta() + new_downvotes, self.a(), self.b())
    }
}

