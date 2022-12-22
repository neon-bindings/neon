use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static POKEDEX: Lazy<Pokedex> = Lazy::new(|| {
    static POKEMON: &str = include_str!("../data/pokemon.json");

    serde_json::from_str(POKEMON).unwrap()
});

pub fn pokedex() -> &'static Pokedex {
    Lazy::force(&POKEDEX)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pokedex {
    pub pokemon: Vec<Pokemon>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pokemon {
    pub id: i64,
    pub num: String,
    pub name: String,
    pub img: String,
    #[serde(rename = "type")]
    pub pokemon_type: Vec<Type>,
    pub height: String,
    pub weight: String,
    pub candy: String,
    pub candy_count: Option<i64>,
    pub egg: Egg,
    pub spawn_chance: f64,
    pub avg_spawns: f64,
    pub spawn_time: String,
    pub multipliers: Option<Vec<f64>>,
    pub weaknesses: Vec<Type>,
    pub next_evolution: Option<Vec<Evolution>>,
    pub prev_evolution: Option<Vec<Evolution>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Evolution {
    pub num: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Egg {
    #[serde(rename = "Not in Eggs")]
    NotInEggs,
    #[serde(rename = "Omanyte Candy")]
    OmanyteCandy,
    #[serde(rename = "10 km")]
    The10Km,
    #[serde(rename = "2 km")]
    The2Km,
    #[serde(rename = "5 km")]
    The5Km,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    Bug,
    Dark,
    Dragon,
    Electric,
    Fairy,
    Fighting,
    Fire,
    Flying,
    Ghost,
    Grass,
    Ground,
    Ice,
    Normal,
    Poison,
    Psychic,
    Rock,
    Steel,
    Water,
}
