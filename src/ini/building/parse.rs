use std::str::FromStr;

use lazy_static::lazy_static;
use const_format::concatcp;
use regex::Regex;

use super::{BuildingType,
            StorageCargoType,
            ParticleType,
            ConstructionPhase,
            ConstructionAutoCost,
            ResourceType,
            Token,
            StrValue,
            QuotedStringParam,
            IdStringParam,
            Point3f,
            Rect,
           };

use super::super::{ParseResult, ParseError};


const RX_REMAINDER: &str = r"($|\s*(.*))";

trait ParseSlice<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> where Self: Sized;
}




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

impl ParseSlice<'_> for Point3f {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        let((x, y, z), src) = <(f32, f32, f32) as ParseSlice>::parse(src)?;
        Ok((Point3f { x, y, z }, src))
    }
}

impl ParseSlice<'_> for Rect {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        let((x1, z1, x2, z2), src) = <(f32, f32, f32, f32) as ParseSlice>::parse(src)?;
        Ok((Rect { x1, z1, x2, z2 }, src))
    }
}

impl ParseSlice<'_> for f32 {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^(-?[0-9]*\.?[0-9]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| f32::from_str(s).map_err(|e| format!("f32 parse failed: {}", e)))
    }
}


impl ParseSlice<'_> for u32 {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([0-9]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| u32::from_str(s).map_err(|e| format!("u32 parse failed: {}", e)))
    }
}


impl<'a> ParseSlice<'a> for QuotedStringParam<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!("(?s)^\"([^\"\\n]+)\"", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Ok(Self(StrValue::Borrowed(s))))
    }
}

impl<'a> ParseSlice<'a> for IdStringParam<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([^[:space:]]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Ok(Self(StrValue::Borrowed(s))))
    }
}



impl BuildingType {
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



impl StorageCargoType {
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


impl ParticleType {
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


impl ConstructionPhase {
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



impl ConstructionAutoCost {
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



impl ResourceType {
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



impl<'a> Token<'a> {

    fn parse(src: &'a str) -> ParseResult<Token<'a>> {
        let (t_type, rest) = chop_token_type(src)?;
        match t_type {
            Self::NAME_STR => 
                QuotedStringParam::parse(rest).map(|(p, rest)| (Self::NameStr(p), rest)),

            Self::NAME => 
                u32::parse(rest).map(|(p, rest)| (Self::Name(p), rest)),

            Self::BUILDING_TYPE =>
                BuildingType::parse(rest).map(|(p, rest)| (Self::BuildingType(p), rest)),

            Self::CIVIL_BUILDING =>
                Ok((Self::CivilBuilding, rest)),

            Self::QUALITY_OF_LIVING =>
                f32::parse(rest).map(|(p, rest)| (Self::QualityOfLiving(p), rest)),

            Self::WORKERS_NEEDED =>
                u32::parse(rest).map(|(p, rest)| (Self::WorkersNeeded(p), rest)),

            Self::CITIZEN_ABLE_SERVE =>
                u32::parse(rest).map(|(p, rest)| (Self::CitizenAbleServe(p), rest)),

            Self::STORAGE =>
                <(StorageCargoType, f32)>::parse(rest).map(|(p, rest)| (Self::Storage(p), rest)),

            Self::CONNECTION_PEDESTRIAN =>
                <(Point3f, Point3f)>::parse(rest).map(|(p, rest)| (Self::ConnectionPedestrian(p), rest)),

            Self::CONNECTION_ROAD =>
                <(Point3f, Point3f)>::parse(rest).map(|(p, rest)| (Self::ConnectionRoad(p), rest)),

            Self::CONNECTION_ROAD_DEAD =>
                Point3f::parse(rest).map(|(p, rest)| (Self::ConnectionRoadDead(p), rest)),

            Self::CONNECTIONS_ROAD_DEAD_SQUARE =>
                <Rect>::parse(rest).map(|(p, rest)| (Self::ConnectionsRoadDeadSquare(p), rest)),

            Self::PARTICLE =>
                <(ParticleType, Point3f, f32, f32)>::parse(rest).map(|(p, rest)| (Self::Particle(p), rest)),

            Self::TEXT_CAPTION =>
                <(Point3f, Point3f)>::parse(rest).map(|(p, rest)| (Self::TextCaption(p), rest)),

            Self::COST_WORK =>
                <(ConstructionPhase, f32)>::parse(rest).map(|(p, rest)| (Self::CostWork(p), rest)),

            Self::COST_WORK_BUILDING_NODE =>
                IdStringParam::parse(rest).map(|(p, rest)| (Self::CostWorkBuildingNode(p), rest)),

            Self::COST_RESOURCE =>
                <(ResourceType, f32)>::parse(rest).map(|(p, rest)| (Self::CostResource(p), rest)),

            Self::COST_RESOURCE_AUTO =>
                <(ConstructionAutoCost, f32)>::parse(rest).map(|(p, rest)| (Self::CostResourceAuto(p), rest)),

            Self::COST_WORK_VEHICLE_STATION =>
                <(Point3f, Point3f)>::parse(rest).map(|(p, rest)| (Self::CostWorkVehicleStation(p), rest)),

            Self::COST_WORK_VEHICLE_STATION_NODE =>
                IdStringParam::parse(rest).map(|(p, rest)| (Self::CostWorkVehicleStationNode(p), rest)),

            _ => Err(format!("Unknown token type: \"${}\"", t_type))
        }
    }
}


lazy_static! {
    static ref RX_SPLIT: Regex = Regex::new(concatcp!("(^?", r"((\s*\r?)|(--[^\n]*)\n)+", r")(\$|end\s*(\r?\n\s*)*)")).unwrap();
}


pub fn parse_tokens_all<'a>(src: &'a str) -> Vec<(&'a str, ParseResult<'a, Token<'a>>)> {
    RX_SPLIT.split(src)
        .filter(|x| !x.is_empty())
        .map(|t_str| (t_str, Token::parse(t_str)))
        .collect()
}

pub fn parse_tokens_strict<'a>(src: &'a str) -> Result<Vec<(&'a str, Token<'a>)>, Vec<(&'a str, ParseError)>> {
    let mut res = Vec::with_capacity(100);
    let mut errors = Vec::with_capacity(0);

    for t_str in RX_SPLIT.split(src).filter(|x| !x.is_empty()) {
        match Token::parse(t_str) {
            Ok((t_val, rest)) => {
                match rest {
                    Some(r) if !r.is_empty() => {
                        errors.push((t_str, format!("Token parsed incomplete. Remaining: {}", r)));
                    },
                    _ => res.push((t_str, t_val))
                }
            },
            Err(e) => {
                errors.push((t_str, e));
            }
        }
    }

    if errors.is_empty() {
        Ok(res)
    } else {
        Err(errors)
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
