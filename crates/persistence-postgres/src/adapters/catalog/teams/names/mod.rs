mod add_team_name;
mod mapper;
mod normalization;
mod row;
mod validation;

pub(in crate::adapters::catalog::teams) use normalization::normalize_team_name;
