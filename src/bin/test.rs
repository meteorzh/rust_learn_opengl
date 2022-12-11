use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;

struct Cacher<I, O, T>
where 
    T: Fn(I) -> O,
{
    calc: T,
    cache: HashMap<I, O>
}


impl<I: Eq + Hash + Copy, O, T> Cacher<I, O, T> 
where
    T: Fn(I) -> O
{
    fn new(calc: T) -> Cacher<I, O, T> {
        Cacher {
            calc,
            cache: HashMap::new()
        }
    }

    fn get(&mut self, n: &I) -> &O {
        self.cache.entry(*n).or_insert((self.calc)(*n))
    }
}

fn main() {
    let mut workout = Cacher::new(|x| x + 1);
    println!("{}", workout.get(&4));
    println!("{}", workout.get(&5));
    println!("{}", workout.get(&3));
    println!("{}", workout.get(&4));
}