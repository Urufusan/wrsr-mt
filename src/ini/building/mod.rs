use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use lazy_static::lazy_static;
use const_format::concatcp;
use regex::Regex;

mod display;

pub enum BuildingType {
    AirplaneGate,
    AirplaneParking,
    AirplaneTower,
    Attraction,
    Broadcast,
    CarDealer,
    CargoStation,
    Church,
    Cityhall,
    ConstructionOffice,
    ConstructionOfficeRail,
    ContainerFacility,
    CoolingTower,
    Customhouse,
    DistributionOffice,
    ElectricExport,
    ElectricImport,
    Engine,
    Factory,
    Farm,
    Field,
    Firestation,
    ForkliftGarage,
    GasStation,
    HeatingEndstation,
    HeatingPlant,
    HeatingSwitch,
    Hospital,
    Hotel,
    Kindergarten,
    Kino,
    Living,
    MineBauxite,
    MineCoal,
    MineGravel,
    MineIron,
    MineOil,
    MineUranium,
    MineWood,
    Monument,
    Parking,
    PassangerStation,
    PollutionMeter,
    Powerplant,
    ProductionLine,
    Pub,
    RailTrafo,
    Raildepo,
    Roaddepo,
    School,
    ShipDock,
    Shop,
    Sport,
    Storage,
    Substation,
    Transformator,
    University,
}


impl BuildingType {
    const AIRPLANE_GATE:            &'static str = "AIRPLANE_GATE";
    const AIRPLANE_PARKING:         &'static str = "AIRPLANE_PARKING";
    const AIRPLANE_TOWER:           &'static str = "AIRPLANE_TOWER";
    const ATTRACTION:               &'static str = "ATTRACTION";
    const BROADCAST:                &'static str = "BROADCAST";
    const CAR_DEALER:               &'static str = "CAR_DEALER";
    const CARGO_STATION:            &'static str = "CARGO_STATION";
    const CHURCH:                   &'static str = "CHURCH";
    const CITYHALL:                 &'static str = "CITYHALL";
    const CONSTRUCTION_OFFICE:      &'static str = "CONSTRUCTION_OFFICE";
    const CONSTRUCTION_OFFICE_RAIL: &'static str = "CONSTRUCTION_OFFICE_RAIL";
    const CONTAINER_FACILITY:       &'static str = "CONTAINER_FACILITY";
    const COOLING_TOWER:            &'static str = "COOLING_TOWER";
    const CUSTOMHOUSE:              &'static str = "CUSTOMHOUSE";
    const DISTRIBUTION_OFFICE:      &'static str = "DISTRIBUTION_OFFICE";
    const ELETRIC_EXPORT:           &'static str = "ELETRIC_EXPORT";
    const ELETRIC_IMPORT:           &'static str = "ELETRIC_IMPORT";
    const ENGINE:                   &'static str = "ENGINE";
    const FACTORY:                  &'static str = "FACTORY";
    const FARM:                     &'static str = "FARM";
    const FIELD:                    &'static str = "FIELD";
    const FIRESTATION:              &'static str = "FIRESTATION";
    const FORKLIFT_GARAGE:          &'static str = "FORKLIFT_GARAGE";
    const GAS_STATION:              &'static str = "GAS_STATION";
    const HEATING_ENDSTATION:       &'static str = "HEATING_ENDSTATION";
    const HEATING_PLANT:            &'static str = "HEATING_PLANT";
    const HEATING_SWITCH:           &'static str = "HEATING_SWITCH";
    const HOSPITAL:                 &'static str = "HOSPITAL";
    const HOTEL:                    &'static str = "HOTEL";
    const KINDERGARTEN:             &'static str = "KINDERGARTEN";
    const KINO:                     &'static str = "KINO";
    const LIVING:                   &'static str = "LIVING";
    const MINE_BAUXITE:             &'static str = "MINE_BAUXITE";
    const MINE_COAL:                &'static str = "MINE_COAL";
    const MINE_GRAVEL:              &'static str = "MINE_GRAVEL";
    const MINE_IRON:                &'static str = "MINE_IRON";
    const MINE_OIL:                 &'static str = "MINE_OIL";
    const MINE_URANIUM:             &'static str = "MINE_URANIUM";
    const MINE_WOOD:                &'static str = "MINE_WOOD";
    const MONUMENT:                 &'static str = "MONUMENT";
    const PARKING:                  &'static str = "PARKING";
    const PASSANGER_STATION:        &'static str = "PASSANGER_STATION";
    const POLLUTION_METER:          &'static str = "POLLUTION_METER";
    const POWERPLANT:               &'static str = "POWERPLANT";
    const PRODUCTION_LINE:          &'static str = "PRODUCTION_LINE";
    const PUB:                      &'static str = "PUB";
    const RAIL_TRAFO:               &'static str = "RAIL_TRAFO";
    const RAILDEPO:                 &'static str = "RAILDEPO";
    const ROADDEPO:                 &'static str = "ROADDEPO";
    const SCHOOL:                   &'static str = "SCHOOL";
    const SHIP_DOCK:                &'static str = "SHIP_DOCK";
    const SHOP:                     &'static str = "SHOP";
    const SPORT:                    &'static str = "SPORT";
    const STORAGE:                  &'static str = "STORAGE";
    const SUBSTATION:               &'static str = "SUBSTATION";
    const TRANSFORMATOR:            &'static str = "TRANSFORMATOR";
    const UNIVERSITY:               &'static str = "UNIVERSITY";

    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::AIRPLANE_GATE            => Some(Self::AirplaneGate),
            Self::AIRPLANE_PARKING         => Some(Self::AirplaneParking),
            Self::AIRPLANE_TOWER           => Some(Self::AirplaneTower),
            Self::ATTRACTION               => Some(Self::Attraction),
            Self::BROADCAST                => Some(Self::Broadcast),
            Self::CAR_DEALER               => Some(Self::CarDealer),
            Self::CARGO_STATION            => Some(Self::CargoStation),
            Self::CHURCH                   => Some(Self::Church),
            Self::CITYHALL                 => Some(Self::Cityhall),
            Self::CONSTRUCTION_OFFICE      => Some(Self::ConstructionOffice),
            Self::CONSTRUCTION_OFFICE_RAIL => Some(Self::ConstructionOfficeRail),
            Self::CONTAINER_FACILITY       => Some(Self::ContainerFacility),
            Self::COOLING_TOWER            => Some(Self::CoolingTower),
            Self::CUSTOMHOUSE              => Some(Self::Customhouse),
            Self::DISTRIBUTION_OFFICE      => Some(Self::DistributionOffice),
            Self::ELETRIC_EXPORT           => Some(Self::ElectricExport),
            Self::ELETRIC_IMPORT           => Some(Self::ElectricImport),
            Self::ENGINE                   => Some(Self::Engine),
            Self::FACTORY                  => Some(Self::Factory),
            Self::FARM                     => Some(Self::Farm),
            Self::FIELD                    => Some(Self::Field),
            Self::FIRESTATION              => Some(Self::Firestation),
            Self::FORKLIFT_GARAGE          => Some(Self::ForkliftGarage),
            Self::GAS_STATION              => Some(Self::GasStation),
            Self::HEATING_ENDSTATION       => Some(Self::HeatingEndstation),
            Self::HEATING_PLANT            => Some(Self::HeatingPlant),
            Self::HEATING_SWITCH           => Some(Self::HeatingSwitch),
            Self::HOSPITAL                 => Some(Self::Hospital),
            Self::HOTEL                    => Some(Self::Hotel),
            Self::KINDERGARTEN             => Some(Self::Kindergarten),
            Self::KINO                     => Some(Self::Kino),
            Self::LIVING                   => Some(Self::Living),
            Self::MINE_BAUXITE             => Some(Self::MineBauxite),
            Self::MINE_COAL                => Some(Self::MineCoal),
            Self::MINE_GRAVEL              => Some(Self::MineGravel),
            Self::MINE_IRON                => Some(Self::MineIron),
            Self::MINE_OIL                 => Some(Self::MineOil),
            Self::MINE_URANIUM             => Some(Self::MineUranium),
            Self::MINE_WOOD                => Some(Self::MineWood),
            Self::MONUMENT                 => Some(Self::Monument),
            Self::PARKING                  => Some(Self::Parking),
            Self::PASSANGER_STATION        => Some(Self::PassangerStation),
            Self::POLLUTION_METER          => Some(Self::PollutionMeter),
            Self::POWERPLANT               => Some(Self::Powerplant),
            Self::PRODUCTION_LINE          => Some(Self::ProductionLine),
            Self::PUB                      => Some(Self::Pub),
            Self::RAIL_TRAFO               => Some(Self::RailTrafo),
            Self::RAILDEPO                 => Some(Self::Raildepo),
            Self::ROADDEPO                 => Some(Self::Roaddepo),
            Self::SCHOOL                   => Some(Self::School),
            Self::SHIP_DOCK                => Some(Self::ShipDock),
            Self::SHOP                     => Some(Self::Shop),
            Self::SPORT                    => Some(Self::Sport),
            Self::STORAGE                  => Some(Self::Storage),
            Self::SUBSTATION               => Some(Self::Substation),
            Self::TRANSFORMATOR            => Some(Self::Transformator),
            Self::UNIVERSITY               => Some(Self::University),
            _ => None
        }
    }
}

impl ParseSlice<'_> for BuildingType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Self::from_str(s).ok_or(format!("Unknown building type '{}'", s)))
    }
}


pub enum StorageCargoType {
    Passanger,
    Cement,
    Covered,
    Gravel,
    Oil,
    Open,
    Cooler,
    Concrete,
    Livestock,
    General,
}


impl StorageCargoType {
    const PASSANGER: &'static str = "RESOURCE_TRANSPORT_PASSANGER";
    const CEMENT:    &'static str = "RESOURCE_TRANSPORT_CEMENT";
    const COVERED:   &'static str = "RESOURCE_TRANSPORT_COVERED";
    const GRAVEL:    &'static str = "RESOURCE_TRANSPORT_GRAVEL";
    const OIL:       &'static str = "RESOURCE_TRANSPORT_OIL";
    const OPEN:      &'static str = "RESOURCE_TRANSPORT_OPEN";
    const COOLER:    &'static str = "RESOURCE_TRANSPORT_COOLER";
    const CONCRETE:  &'static str = "RESOURCE_TRANSPORT_CONCRETE";
    const LIVESTOCK: &'static str = "RESOURCE_TRANSPORT_LIVESTOCK";
    const GENERAL:   &'static str = "RESOURCE_TRANSPORT_GENERAL";

    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::PASSANGER => Some(Self::Passanger),
            Self::CEMENT    => Some(Self::Cement),
            Self::COVERED   => Some(Self::Covered),
            Self::GRAVEL    => Some(Self::Gravel),
            Self::OIL       => Some(Self::Oil),
            Self::OPEN      => Some(Self::Open),
            Self::COOLER    => Some(Self::Cooler),
            Self::CONCRETE  => Some(Self::Concrete),
            Self::LIVESTOCK => Some(Self::Livestock),
            Self::GENERAL   => Some(Self::General),
            _ => None
        }
    }
}

impl ParseSlice<'_> for StorageCargoType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| StorageCargoType::from_str(s).ok_or(format!("Unknown storage cargo type '{}'", s)))
    }
}


pub enum ParticleType {
    ResidentialHeating,
    BigBlack,
    MediumBlack,
    SmallBlack,
    BigGray,
    MediumGray,
    SmallGray,
    BigWhite,
    MediumWhite,
    SmallWhite,
}

impl ParticleType {
    const RESIDENTIAL_HEATING : &'static str = "residential_heating";
    const FACTORY_BIG_BLACK   : &'static str = "factory_big_black";
    const FACTORY_MEDIUM_BLACK: &'static str = "factory_medium_black";
    const FACTORY_SMALL_BLACK : &'static str = "factory_small_black";
    const FACTORY_BIG_GRAY    : &'static str = "factory_big_gray";
    const FACTORY_MEDIUM_GRAY : &'static str = "factory_medium_gray";
    const FACTORY_SMALL_GRAY  : &'static str = "factory_small_gray";
    const FACTORY_BIG_WHITE   : &'static str = "factory_big_white";
    const FACTORY_MEDIUM_WHITE: &'static str = "factory_medium_white";
    const FACTORY_SMALL_WHITE : &'static str = "factory_small_white";

    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::RESIDENTIAL_HEATING  => Some(Self::ResidentialHeating),
            Self::FACTORY_BIG_BLACK    => Some(Self::BigBlack),
            Self::FACTORY_MEDIUM_BLACK => Some(Self::MediumBlack),
            Self::FACTORY_SMALL_BLACK  => Some(Self::SmallBlack),
            Self::FACTORY_BIG_GRAY     => Some(Self::BigGray),
            Self::FACTORY_MEDIUM_GRAY  => Some(Self::MediumGray),
            Self::FACTORY_SMALL_GRAY   => Some(Self::SmallGray),
            Self::FACTORY_BIG_WHITE    => Some(Self::BigWhite),
            Self::FACTORY_MEDIUM_WHITE => Some(Self::MediumWhite),
            Self::FACTORY_SMALL_WHITE  => Some(Self::SmallWhite),
            _ => None
        }
    }
}

impl ParseSlice<'_> for ParticleType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([a-z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| ParticleType::from_str(s).ok_or(format!("Unknown particle type '{}'", s)))
    }
}


pub enum ConstructionPhase {
    Groundworks,
    BoardsLaying,
    BricksLaying,
    SkeletonCasting,
    SteelLaying,
    PanelsLaying,
    RooftopBuilding,
    WireLaying,
    Tunneling,
}


impl ConstructionPhase {
    const GROUNDWORKS:      &'static str = "SOVIET_CONSTRUCTION_GROUNDWORKS";
    const BOARDS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_BOARDS_LAYING";
    const BRICKS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_BRICKS_LAYING";
    const SKELETON_CASTING: &'static str = "SOVIET_CONSTRUCTION_SKELETON_CASTING";
    const STEEL_LAYING:     &'static str = "SOVIET_CONSTRUCTION_STEEL_LAYING";
    const PANELS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_PANELS_LAYING";
    const ROOFTOP_BUILDING: &'static str = "SOVIET_CONSTRUCTION_ROOFTOP_BUILDING";
    const WIRE_LAYING:      &'static str = "SOVIET_CONSTRUCTION_WIRE_LAYING";
    const TUNNELING:        &'static str = "SOVIET_CONSTRUCTION_TUNNELING";

    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::GROUNDWORKS      => Some(Self::Groundworks),
            Self::BOARDS_LAYING    => Some(Self::BoardsLaying),
            Self::BRICKS_LAYING    => Some(Self::BricksLaying),
            Self::SKELETON_CASTING => Some(Self::SkeletonCasting),
            Self::STEEL_LAYING     => Some(Self::SteelLaying),
            Self::PANELS_LAYING    => Some(Self::PanelsLaying),
            Self::ROOFTOP_BUILDING => Some(Self::RooftopBuilding),
            Self::WIRE_LAYING      => Some(Self::WireLaying),
            Self::TUNNELING        => Some(Self::Tunneling),
            _ => None
        }
    }
}

impl ParseSlice<'_> for ConstructionPhase {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| ConstructionPhase::from_str(s).ok_or(format!("Unknown construction phase '{}'", s)))
    }
}


pub enum ConstructionAutoCost {
    Ground,
    GroundAsphalt,
    WallConcrete,
    WallPanels,
    WallBrick,
    WallSteel,
    WallWood,
    TechSteel,
    ElectroSteel,
    RoofWoodBrick,
    RoofSteel,
    RoofWoodSteel
}


impl ConstructionAutoCost {
    const GROUND:          &'static str = "ground";
    const GROUND_ASPHALT:  &'static str = "ground_asphalt";
    const WALL_CONCRETE:   &'static str = "wall_concrete";
    const WALL_PANELS:     &'static str = "wall_panels";
    const WALL_BRICK:      &'static str = "wall_brick";
    const WALL_STEEL:      &'static str = "wall_steel";
    const WALL_WOOD:       &'static str = "wall_wood";
    const TECH_STEEL:      &'static str = "tech_steel";
    const ELECTRO_STEEL:   &'static str = "electro_steel";
    const ROOF_WOOD_BRICK: &'static str = "roof_woodbrick";
    const ROOF_STEEL:      &'static str = "roof_steel";
    const ROOF_WOOD_STEEL: &'static str = "roof_woodsteel";

    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::GROUND          => Some(Self::Ground),
            Self::GROUND_ASPHALT  => Some(Self::GroundAsphalt),
            Self::WALL_CONCRETE   => Some(Self::WallConcrete),
            Self::WALL_PANELS     => Some(Self::WallPanels),
            Self::WALL_BRICK      => Some(Self::WallBrick),
            Self::WALL_STEEL      => Some(Self::WallSteel),
            Self::WALL_WOOD       => Some(Self::WallWood),
            Self::TECH_STEEL      => Some(Self::TechSteel),
            Self::ELECTRO_STEEL   => Some(Self::ElectroSteel),
            Self::ROOF_WOOD_BRICK => Some(Self::RoofWoodBrick),
            Self::ROOF_STEEL      => Some(Self::RoofSteel),
            Self::ROOF_WOOD_STEEL => Some(Self::RoofWoodSteel),
            _ => None
        }
    }
}

impl ParseSlice<'_> for ConstructionAutoCost {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([a-z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| ConstructionAutoCost::from_str(s).ok_or(format!("Unknown construction auto cost '{}'", s)))
    }
}


pub enum ResourceType {
    Alcohol,
    Alumina,
    Aluminium,
    Asphalt,
    Bauxite,
    Boards,
    Bricks,
    Chemicals,
    Clothes,
    Concrete,
    ElectroComponents,
    Electricity,
    Electronics,
    Food,
    Gravel,
    MechComponents,
    Meat,
    NuclearFuel,
    Oil,
    Crops,
    PrefabPanels,
    Steel,
    UF6,
    Uranium,
    Wood,
    Workers,
    Yellowcake,
}


impl ResourceType {
    const ALCOHOL:      &'static str = "alcohol";
    const ALUMINA:      &'static str = "alumina";
    const ALUMINIUM:    &'static str = "aluminium";
    const ASPHALT:      &'static str = "asphalt";
    const BAUXITE:      &'static str = "bauxite";
    const BOARDS:       &'static str = "boards";
    const BRICKS:       &'static str = "bricks";
    const CHEMICALS:    &'static str = "chemicals";
    const CLOTHES:      &'static str = "clothes";
    const CONCRETE:     &'static str = "concrete";
    const ELECTRO_COMP: &'static str = "ecomponents";
    const ELECTRICITY:  &'static str = "eletric";
    const ELECTRONICS:  &'static str = "eletronics";
    const FOOD:         &'static str = "food";
    const GRAVEL:       &'static str = "gravel";
    const MECH_COMP:    &'static str = "mcomponents";
    const MEAT:         &'static str = "meat";
    const NUCLEAR_FUEL: &'static str = "nuclearfuel";
    const OIL:          &'static str = "oil";
    const CROPS:        &'static str = "plants";
    const PREFABS:      &'static str = "prefabpanels";
    const STEEL:        &'static str = "steel";
    const UF_6:         &'static str = "uf6";
    const URANIUM:      &'static str = "uranium";
    const WOOD:         &'static str = "wood";
    const WORKERS:      &'static str = "workers";
    const YELLOWCAKE:   &'static str = "yellowcake";

    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::ALCOHOL      => Some(Self::Alcohol),
            Self::ALUMINA      => Some(Self::Alumina),
            Self::ALUMINIUM    => Some(Self::Aluminium),
            Self::ASPHALT      => Some(Self::Asphalt),
            Self::BAUXITE      => Some(Self::Bauxite),
            Self::BOARDS       => Some(Self::Boards),
            Self::BRICKS       => Some(Self::Bricks),
            Self::CHEMICALS    => Some(Self::Chemicals),
            Self::CLOTHES      => Some(Self::Clothes),
            Self::CONCRETE     => Some(Self::Concrete),
            Self::ELECTRO_COMP => Some(Self::ElectroComponents),
            Self::ELECTRICITY  => Some(Self::Electricity),
            Self::ELECTRONICS  => Some(Self::Electronics),
            Self::FOOD         => Some(Self::Food),
            Self::GRAVEL       => Some(Self::Gravel),
            Self::MECH_COMP    => Some(Self::MechComponents),
            Self::MEAT         => Some(Self::Meat),
            Self::NUCLEAR_FUEL => Some(Self::NuclearFuel),
            Self::OIL          => Some(Self::Oil),
            Self::CROPS        => Some(Self::Crops),
            Self::PREFABS      => Some(Self::PrefabPanels),
            Self::STEEL        => Some(Self::Steel),
            Self::UF_6         => Some(Self::UF6),
            Self::URANIUM      => Some(Self::Uranium),
            Self::WOOD         => Some(Self::Wood),
            Self::WORKERS      => Some(Self::Workers),
            Self::YELLOWCAKE   => Some(Self::Yellowcake),
            _ => None
        }
    }
}

impl ParseSlice<'_> for ResourceType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([a-z0-9_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| ResourceType::from_str(s).ok_or(format!("Unknown resource type '{}'", s)))
    }
}


pub enum Token<'a> {

    NameStr(QuotedStringParam<'a>),
//  Name(u32),

    BuildingType(BuildingType),

    Storage((StorageCargoType, f32)),

    ConnectionPedestrian((Point3f, Point3f)),

    Particle((ParticleType, Point3f, f32, f32)),

    CostWork((ConstructionPhase, f32)),

    CostWorkBuildingNode(IdStringParam<'a>),
    CostResource((ResourceType, f32)),
    CostResourceAuto((ConstructionAutoCost, f32)),
    CostWorkVehicleStation(IdStringParam<'a>),

}

type Point3f = (f32, f32, f32);

impl<'a> Token<'a> {
    const NAME_STR:                  &'static str = "NAME_STR";
    const NAME:                      &'static str = "NAME";
    const BUILDING_TYPE:             &'static str = "TYPE_";
    const STORAGE:                   &'static str = "STORAGE";
    const CONNECTION_PEDESTRIAN:     &'static str = "CONNECTION_PEDESTRIAN";
    const PARTICLE:                  &'static str = "PARTICLE";
    const COST_WORK:                 &'static str = "COST_WORK";
    const COST_WORK_BUILDING_NODE:   &'static str = "COST_WORK_BUILDING_NODE";
    const COST_RESOURCE:             &'static str = "COST_RESOURCE";
    const COST_RESOURCE_AUTO:        &'static str = "COST_RESOURCE_AUTO";
    const COST_WORK_VEHICLE_STATION: &'static str = "COST_WORK_VEHICLE_STATION_ACCORDING_NODE";

    fn parse(src: &'a str) -> ParseResult<Token<'a>> {
        let (t_type, rest) = chop_token_type(src)?;
        match t_type {
            Self::NAME_STR => 
                QuotedStringParam::parse(rest).map(|(p, rest)| (Self::NameStr(p), rest)),

//            Self::NAME => 
//                TokenParams0::parse(rest).map(|(p, rest)| (Self::Name(p), rest)),

            Self::BUILDING_TYPE =>
                BuildingType::parse(rest).map(|(p, rest)| (Self::BuildingType(p), rest)),

            Self::STORAGE =>
                <(StorageCargoType, f32)>::parse(rest).map(|(p, rest)| (Self::Storage(p), rest)),

            Self::CONNECTION_PEDESTRIAN =>
                <(Point3f, Point3f)>::parse(rest).map(|(p, rest)| (Self::ConnectionPedestrian(p), rest)),

            Self::PARTICLE =>
                <(ParticleType, Point3f, f32, f32)>::parse(rest).map(|(p, rest)| (Self::Particle(p), rest)),

            Self::COST_WORK =>
                <(ConstructionPhase, f32)>::parse(rest).map(|(p, rest)| (Self::CostWork(p), rest)),

            Self::COST_WORK_BUILDING_NODE =>
                IdStringParam::parse(rest).map(|(p, rest)| (Self::CostWorkBuildingNode(p), rest)),

            Self::COST_RESOURCE =>
                <(ResourceType, f32)>::parse(rest).map(|(p, rest)| (Self::CostResource(p), rest)),

            Self::COST_RESOURCE_AUTO =>
                <(ConstructionAutoCost, f32)>::parse(rest).map(|(p, rest)| (Self::CostResourceAuto(p), rest)),

            Self::COST_WORK_VEHICLE_STATION =>
                IdStringParam::parse(rest).map(|(p, rest)| (Self::CostWorkVehicleStation(p), rest)),

            _ => Err(format!("Unknown token type: [{}]", t_type))
        }
    }
}

const RX_REMAINDER: &str = r"($|\s*(.*))";

pub type ParseError = String;
pub type ParseResult<'a, T> = Result<(T, Option<&'a str>), ParseError>;

fn parse_param<'a, T, F: Fn(&'a str) -> Result<T, ParseError>>(src: Option<&'a str>, rx: &Regex, f: F) -> ParseResult<'a, T> {
    let src = src.ok_or(String::from("Parse param failed: no data"))?;

    match rx.captures(src) {
        Some(caps) => {
            // allow panic here: this should not happen with valid regex:
            let t = caps.get(1).expect("Regex is broken").as_str();
            let rest = caps.get(3).map(|x| x.as_str());

            let v = f(t)?;
            Ok((v, rest))
        },
        None => Err(String::from("Parse param failed (no regex match)"))
    }
}


pub trait ParseSlice<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> where Self: Sized;
}

impl ParseSlice<'_> for f32 {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^(-?[0-9]*\.?[0-9]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| f32::from_str(s).map_err(|e| format!("Float parse failed: {}", e)))
    }
}

impl<'a, T1, T2> ParseSlice<'a> for (T1, T2)
where T1: ParseSlice<'a>,
      T2: ParseSlice<'a>
{
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        let (t1, src) = T1::parse(src)?;
        let (t2, src) = T2::parse(src)?;
        Ok(((t1, t2), src))
    }
}

impl<'a, T1, T2, T3> ParseSlice<'a> for (T1, T2, T3)
where T1: ParseSlice<'a>,
      T2: ParseSlice<'a>,
      T3: ParseSlice<'a>
{
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        let (t1, src) = T1::parse(src)?;
        let (t2, src) = T2::parse(src)?;
        let (t3, src) = T3::parse(src)?;
        Ok(((t1, t2, t3), src))
    }
}

impl<'a, T1, T2, T3, T4> ParseSlice<'a> for (T1, T2, T3, T4)
where T1: ParseSlice<'a>,
      T2: ParseSlice<'a>,
      T3: ParseSlice<'a>,
      T4: ParseSlice<'a>
{
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        let (t1, src) = T1::parse(src)?;
        let (t2, src) = T2::parse(src)?;
        let (t3, src) = T3::parse(src)?;
        let (t4, src) = T4::parse(src)?;
        Ok(((t1, t2, t3, t4), src))
    }
}


pub enum StrValue<'a> {
    Borrowed(&'a str),
    Owned(String),
}


pub struct QuotedStringParam<'a>(StrValue<'a>);


impl<'a> ParseSlice<'a> for QuotedStringParam<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!("(?s)^\"([^\"\\n]+)\"", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Ok(Self(StrValue::Borrowed(s))))
    }
}

pub struct IdStringParam<'a>(StrValue<'a>);

impl<'a> ParseSlice<'a> for IdStringParam<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([^[:space:]]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Ok(Self(StrValue::Borrowed(s))))
    }
}



fn chop_token_type<'a>(src: &'a str) -> ParseResult<&'a str> {
    lazy_static! {
        static ref RX_TYPE: Regex = Regex::new(r"(?s)^(TYPE_|[A-Z_]+)($|\s*(.*))").unwrap();
    }

    match RX_TYPE.captures(src) {
        Some(caps) => {
            // allow panic here: this should not happen with valid regex:
            let t = caps.get(1).expect("Regex is broken").as_str();
            let rest = caps.get(3).map(|x| x.as_str());
            Ok((t, rest))
        },
        None => Err(format!("Cannot parse token type from this: [{}]", src))
    }
}


fn get_tokens<'a>(src: &'a str) -> Vec<&'a str> {
    const RX_SKIP_LINE: &str = r"((--[^\r\n]*)?\s*\r?\n)+";

    lazy_static! {
        static ref RX_SPLIT: Regex = Regex::new(concatcp!("(", RX_SKIP_LINE, r"|^", RX_SKIP_LINE, r")(\$|end\s*(\r?\n\s*)*)")).unwrap();
    }

    RX_SPLIT.split(src).filter(|x| !x.is_empty()).collect()
}


pub fn do_stuff() {
    let file = fs::read_to_string(r"z:\building.ini").unwrap();

    let tokens = get_tokens(&file);

    for t in tokens {
        //println!("token: [{}]", t);
        match Token::parse(t) {
            Ok((t, rest)) => {
                print!("{}", t);
                if let Some(rest) = rest {
                    print!(" [remainder: {:?}]", rest);
                }
                println!();
            },
            Err(e) => println!("Error: {}, token: [{}]", e, t),
        }
    }

    //println!("OK");
}
