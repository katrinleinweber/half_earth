use crate::state::State;
use crate::npcs::NPCRelation;
use crate::kinds::{Resource, Output};
use crate::production::ProcessFeature;
use crate::projects::Status as ProjectStatus;
use super::{WorldVariable, LocalVariable, PlayerVariable};

#[derive(Debug, Clone)]
pub enum Condition {
    LocalVariable(LocalVariable, Comparator, f32),
    WorldVariable(WorldVariable, Comparator, f32),
    PlayerVariable(PlayerVariable, Comparator, f32),
    ProcessMixShare(usize, Comparator, f32),
    ProcessMixShareFeature(ProcessFeature, Comparator, f32),
    ResourcePressure(Resource, Comparator, f32),
    ResourceDemandGap(Resource, Comparator, f32),
    OutputDemandGap(Output, Comparator, f32),
    ProjectStatus(usize, ProjectStatus),
    RunsPlayed(Comparator, usize),
    RegionFlag(String),
    NPCRelationship(usize, NPCRelation),
}


impl Condition {
    pub fn eval(&self, state: &State, region_id: Option<usize>) -> bool {
        match self {
            Condition::LocalVariable(var, comp, other_val) => {
                if let Some(id) = region_id {
                    let region = &state.world.regions[id];
                    let val = match var {
                        LocalVariable::Population => region.population,
                        LocalVariable::Outlook => region.outlook,
                        LocalVariable::Habitability => region.habitability(),
                    };
                    comp.eval(val, *other_val)
                } else {
                    false
                }
            },
            Condition::WorldVariable(var, comp, other_val) => {
                let val = match var {
                    WorldVariable::Year => state.world.year as f32,
                    WorldVariable::Population => state.world.population(),
                    WorldVariable::PopulationGrowth => state.world.population_growth_modifier,
                    WorldVariable::Emissions => state.world.emissions(),
                    WorldVariable::ExtinctionRate => state.world.extinction_rate,
                    WorldVariable::Outlook => state.world.outlook(),
                    WorldVariable::Temperature => state.world.temperature,
                    WorldVariable::WaterStress => state.world.water_stress,
                    WorldVariable::SeaLevelRise => state.world.sea_level_rise,
                    WorldVariable::Precipitation => state.world.precipitation,
                };
                comp.eval(val, *other_val)
            },
            Condition::PlayerVariable(var, comp, other_val) => {
                let val = match var {
                    PlayerVariable::PoliticalCapital => state.political_capital as f32,
                    PlayerVariable::MalthusianPoints => state.malthusian_points as f32,
                    PlayerVariable::FALCPoints => state.falc_points as f32,
                    PlayerVariable::HESPoints => state.hes_points as f32,
                };
                comp.eval(val, *other_val)
            },
            Condition::ProcessMixShare(id, comp, other_val) => {
                let val = state.processes[*id].mix_share;
                comp.eval(val, *other_val)
            },
            Condition::ProcessMixShareFeature(feat, comp, other_val) => {
                let val = state.processes.iter().filter(|p| p.features.contains(feat)).map(|p| p.mix_share).sum();
                comp.eval(val, *other_val)
            },
            Condition::ResourcePressure(resource, comp, other_val) => {
                let val = state.resources[*resource]/state.resources_demand[*resource];
                comp.eval(val, *other_val)
            },
            Condition::ResourceDemandGap(resource, comp, other_val) => {
                let available = state.resources[*resource];
                let demand = state.resources_demand[*resource];
                let val = (available - demand)/demand;
                comp.eval(val, *other_val)
            },
            Condition::OutputDemandGap(output, comp, other_val) => {
                let available = state.produced[*output];
                let demand = state.output_demand[*output];
                let val = (available - demand)/demand;
                comp.eval(val, *other_val)
            },
            Condition::RunsPlayed(comp, runs) => {
                comp.eval(state.runs as f32, *runs as f32)
            },
            Condition::ProjectStatus(id, status) => {
                state.projects[*id].status == *status
            },
            Condition::NPCRelationship(id, relation) => {
                state.npcs[*id].relation() == *relation
            },
            Condition::RegionFlag(flag) => {
                if let Some(id) = region_id {
                    let region = &state.world.regions[id];
                    region.flags.contains(flag)
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Comparator {
    Less,
    LessEqual,
    Equal,
    NotEqual,
    GreaterEqual,
    Greater
}

impl Comparator {
    fn eval(&self, a: f32, b: f32) -> bool {
        match self {
            Comparator::Less => a < b,
            Comparator::LessEqual => a <= b,
            Comparator::Equal => a == b,
            Comparator::NotEqual => a != b,
            Comparator::GreaterEqual => a >= b,
            Comparator::Greater => a > b,
        }
    }
}
