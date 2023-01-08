use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Person {
    pub name: String,
    pub group: String,
}

impl Person {
    pub fn new(name: &str, group: &str) -> Self {
        Self {
            name: name.into(),
            group: group.into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Pair {
    pub giver: Person,
    pub receiver: Person,
}

#[derive(Serialize, Deserialize)]
pub struct Hat {
    givers: Vec<Person>,
    receivers: Vec<Person>,
}

#[derive(Debug)]
pub enum DrawError {
    NoGivers,
    NoValidReceiver,
}

impl Hat {
    pub fn with_people(list: Vec<Person>) -> Self {
        Self {
            givers: list.iter().map(Clone::clone).collect(),
            receivers: list,
        }
    }

    pub fn draw_name<F>(&mut self, validate_pair: F) -> Result<Pair, DrawError>
    where
        F: Fn(&Person, &Person) -> bool,
    {
        let mut rng = rand::thread_rng();
        let Some(giver) = self.givers.pop() else { return Err(DrawError::NoGivers); };

        let start_idx = rng.gen_range(0..self.receivers.len());

        for offset in 0..self.receivers.len() {
            let idx = (start_idx + offset) % self.receivers.len();
            let receiver = &self.receivers[idx];

            if !validate_pair(&giver, receiver) {
                continue;
            }

            let receiver = self.receivers.remove(idx);

            if self.valid_solution_exists(&validate_pair) {
                let pair = Pair {
                    giver: giver.clone(),
                    receiver,
                };
                return Ok(pair);
            }

            //fixup on failure
            self.receivers.insert(idx, receiver);
        }

        Err(DrawError::NoValidReceiver)
    }

    fn valid_solution_exists<F>(&self, validate_pair: &F) -> bool
    where
        F: Fn(&Person, &Person) -> bool,
    {
        let mut givers = self.givers.iter().collect();
        let mut receivers = self.receivers.iter().collect();
        self.valid_solution_exists_inner(&mut givers, &mut receivers, validate_pair)
    }

    fn valid_solution_exists_inner<F>(
        &self,
        givers: &mut Vec<&Person>,
        receivers: &mut Vec<&Person>,
        validate_pair: &F,
    ) -> bool
    where
        F: Fn(&Person, &Person) -> bool,
    {
        //success if there are no more givers to assign
        let Some(giver) = givers.pop() else { return true; };

        let iter = receivers
            .iter()
            .enumerate()
            .filter(|(_, r)| validate_pair(giver, r))
            .map(|o| o.0)
            .collect::<Vec<_>>();

        for idx in iter {
            let receiver = receivers.remove(idx);

            if self.valid_solution_exists_inner(givers, receivers, validate_pair) {
                return true;
            }

            receivers.insert(idx, receiver);
        }

        givers.push(giver);

        false
    }
}
