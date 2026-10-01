mod input_quality;
mod lineups;
mod report;
mod workflow;

pub(crate) use workflow::execute;

#[cfg(test)]
mod tests;
