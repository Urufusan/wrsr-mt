use std::str::FromStr;

use lazy_static::lazy_static;
use const_format::concatcp;
use regex::Regex;

use super::{BuildingType,
            BuildingSubtype,
            StorageCargoType,
            ParticleType,
            ConstructionPhase,
            ConstructionAutoCost,
            ResourceType,
            ResourceVisualization,
            Token,

            StrValue,
            QuotedStringParam,
            IdStringParam,
            Point3f,
            Rect,

            Tagged2Points,
            Connection2PType,
            Connection1PType,
            AirplaneStationType,
            AttractionType,
            ResourceSourceType,
           };

use super::super::{ParseResult, ParseError};


macro_rules! parse_from {
    ($src:ident, $t:ty, $id:ident) => {
        <$t>::parse($src).map(|(p, rest)| (Self::$id(p), rest))
    };
}


impl<'a> Token<'a> {

    fn parse(src: &'a str) -> ParseResult<Self> {
        lazy_static! {
            static ref RX_TYPE: Regex = Regex::new(concatcp!(
                r"(?s)^(", 
                Token::CONNECTION,       "|",
                Token::BUILDING_TYPE,    "|",
                Token::BUILDING_SUBTYPE, "|",
                Token::AIRPLANE_STATION, "|",
                Token::ATTRACTION_TYPE,  "|",
                Token::RESOURCE_SOURCE,  "|",
                r"[A-Z_]+)($|\s*(.*))")).unwrap();
        }
    
        let (t_type, rest) = chop_param(Some(src), &RX_TYPE).map_err(|e| format!("Cannot parse token type: {}", e))?;
        match t_type {
            Self::NAME_STR           => parse_from!(rest, QuotedStringParam, NameStr),
            Self::NAME               => parse_from!(rest, u32, Name),

            Self::BUILDING_TYPE      => parse_from!(rest, BuildingType, BuildingType),
            Self::BUILDING_SUBTYPE   => parse_from!(rest, BuildingSubtype, BuildingSubtype),

            Self::HEATING_ENABLE     => Ok((Self::HeatEnable, rest)),
            Self::HEATING_DISABLE    => Ok((Self::HeatDisable, rest)),
            Self::CIVIL_BUILDING     => Ok((Self::CivilBuilding, rest)),
            Self::MONUMENT_ENABLE_TRESPASSING => Ok((Self::MonumentTresspassing, rest)),
            Self::QUALITY_OF_LIVING  => parse_from!(rest, f32, QualityOfLiving),

            Self::WORKERS_NEEDED         => parse_from!(rest, u32, WorkersNeeded),
            Self::PROFESSORS_NEEDED      => parse_from!(rest, u32, ProfessorsNeeded),
            Self::CITIZEN_ABLE_SERVE     => parse_from!(rest, u32, CitizenAbleServe),
            Self::CONSUMPTION            => parse_from!(rest, (ResourceType, f32), Consumption),
            Self::CONSUMPTION_PER_SECOND => parse_from!(rest, (ResourceType, f32), ConsumptionPerSecond),
            Self::PRODUCTION             => parse_from!(rest, (ResourceType, f32), Production),
            Self::PRODUCTION_SUN         => parse_from!(rest, f32, ProductionSun),
            Self::PRODUCTION_WIND        => parse_from!(rest, f32, ProductionWind),
            Self::SEASONAL_TEMP_MIN      => parse_from!(rest, f32, SeasonalTempMin),
            Self::SEASONAL_TEMP_MAX      => parse_from!(rest, f32, SeasonalTempMax),

            Self::ELE_CONSUM_LIGHTING_WORKER_FACTOR => parse_from!(rest, f32, EleConsumLightingWorkerFactor),
            Self::ELE_CONSUM_LIVING_WORKER_FACTOR   => parse_from!(rest, f32, EleConsumLivingWorkerFactor),
            Self::ELE_CONSUM_LOADING_FIXED          => parse_from!(rest, f32, EleConsumLoadingFixed), Self::ELE_CONSUM_UNLOADING_FIXED        => parse_from!(rest, f32, EleConsumUnloadingFixed),
            Self::NO_ELE_FACTOR                     => parse_from!(rest, f32, NoEleFactor),
            Self::NO_ELE_FACTOR_NIGHT               => parse_from!(rest, f32, NoEleFactorNight),
            Self::NO_HEAT_FACTOR                    => parse_from!(rest, f32, NoHeatFactor),

            Self::ENGINE_SPEED           => parse_from!(rest, f32, EngineSpeed),
            Self::CABLEWAY_HEAVY         => Ok((Self::CablewayHeavy, rest)),
            Self::CABLEWAY_LIGHT         => Ok((Self::CablewayLight, rest)),
            Self::RESOURCE_SOURCE        => parse_from!(rest, ResourceSourceType, ResourceSource),

            Self::STORAGE                       => parse_from!(rest, (StorageCargoType, f32), Storage),
            Self::STORAGE_SPECIAL               => parse_from!(rest, (StorageCargoType, f32, ResourceType), StorageSpecial),
            Self::STORAGE_FUEL                  => parse_from!(rest, (StorageCargoType, f32), StorageFuel),
            Self::STORAGE_EXPORT                => parse_from!(rest, (StorageCargoType, f32), StorageExport),
            Self::STORAGE_IMPORT                => parse_from!(rest, (StorageCargoType, f32), StorageImport),
            Self::STORAGE_IMPORT_CARPLANT       => parse_from!(rest, (StorageCargoType, f32), StorageImportCarplant),
            Self::STORAGE_EXPORT_SPECIAL        => parse_from!(rest, (StorageCargoType, f32, ResourceType), StorageExportSpecial),
            Self::STORAGE_IMPORT_SPECIAL        => parse_from!(rest, (StorageCargoType, f32, ResourceType), StorageImportSpecial),
            Self::STORAGE_DEMAND_BASIC          => parse_from!(rest, (StorageCargoType, f32), StorageDemandBasic),
            Self::STORAGE_DEMAND_MEDIUMADVANCED => parse_from!(rest, (StorageCargoType, f32), StorageDemandMediumAdvanced),
            Self::STORAGE_DEMAND_ADVANCED       => parse_from!(rest, (StorageCargoType, f32), StorageDemandAdvanced),
            Self::STORAGE_DEMAND_HOTEL          => parse_from!(rest, (StorageCargoType, f32), StorageDemandHotel),
            Self::STORAGE_PACK_FROM             => parse_from!(rest, u32, StoragePackFrom),
            Self::STORAGE_UNPACK_TO             => parse_from!(rest, u32, StorageUnpackTo),
            Self::STORAGE_LIVING_AUTO           => parse_from!(rest, IdStringParam, StorageLivingAuto),

            Self::VEHICLE_LOADING_FACTOR   => parse_from!(rest, f32, VehicleLoadingFactor),
            Self::VEHICLE_UNLOADING_FACTOR => parse_from!(rest, f32, VehicleUnloadingFactor),

            
            Self::ROAD_VEHICLE_NOT_FLIP     => Ok((Self::RoadNotFlip, rest)),
            Self::ROAD_VEHICLE_ELECTRIC     => Ok((Self::RoadElectric, rest)),
            Self::VEHICLE_CANNOT_SELECT     => Ok((Self::VehicleCannotSelect, rest)),
            Self::LONG_TRAINS               => Ok((Self::LongTrains, rest)),

            Self::WORKING_VEHICLES_NEEDED                    => parse_from!(rest, u32, WorkingVehiclesNeeded),
            Self::VEHICLE_STATION                            => parse_from!(rest, (Point3f, Point3f), VehicleStation),
            Self::VEHICLE_STATION_NOT_BLOCK                  => Ok((Self::VehicleStationNotBlock, rest)),
            Self::VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT     => parse_from!(rest, Point3f, VehicleStationNotBlockDetourPoint),
            Self::VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT_PID => parse_from!(rest, (u32, Point3f), VehicleStationNotBlockDetourPointPid),

            Self::VEHICLE_PARKING                            => parse_from!(rest, (Point3f, Point3f), VehicleParking),
            Self::VEHICLE_PARKING_ADVANCED_POINT             => parse_from!(rest, Point3f, VehicleParkingAdvancedPoint),
            Self::VEHICLE_PARKING_ADVANCED_POINT_PID         => parse_from!(rest, (u32, Point3f), VehicleParkingAdvancedPointPid),
            Self::VEHICLE_PARKING_PERSONAL                   => parse_from!(rest, (Point3f, Point3f), VehicleParkingPersonal),

            Self::AIRPLANE_STATION                => parse_from!(rest, Tagged2Points::<AirplaneStationType>, AirplaneStation),
            Self::HELIPORT_STATION                => parse_from!(rest, (Point3f, Point3f), HeliportStation),
            Self::SHIP_STATION                    => parse_from!(rest, (Point3f, Point3f), ShipStation),
            Self::HELIPORT_AREA                   => parse_from!(rest, f32, HeliportArea),
            Self::HARBOR_OVER_TERRAIN_FROM        => parse_from!(rest, f32, HarborTerrainFrom),
            Self::HARBOR_OVER_WATER_FROM          => parse_from!(rest, f32, HarborWaterFrom),
            Self::HARBOR_EXTEND_WHEN_BULDING      => parse_from!(rest, f32, HarborExtendWhenBuilding),

            Self::CONNECTION => Self::parse_connection(rest),

            Self::CONNECTIONS_SPACE               => parse_from!(rest, Rect, ConnectionsSpace),
            Self::CONNECTIONS_ROAD_DEAD_SQUARE    => parse_from!(rest, Rect, ConnectionsRoadDeadSquare),
            Self::CONNECTIONS_AIRPORT_DEAD_SQUARE => parse_from!(rest, Rect, ConnectionsAirportDeadSquare),
            Self::CONNECTIONS_WATER_DEAD_SQUARE   => parse_from!(rest, (f32, Rect), ConnectionsWaterDeadSquare),
            Self::OFFSET_CONNECTION_XYZW          => parse_from!(rest, (u32, Point3f), OffsetConnection),

            Self::ATTRACTION_TYPE                  => parse_from!(rest, (AttractionType, u32), AttractionType),
            Self::ATTRACTION_REMEMBER_USAGE        => Ok((Self::AttractionRememberUsage, rest)),
            Self::ATTRACTIVE_SCORE                 => parse_from!(rest, f32, AttractiveScoreBase),
            Self::ATTRACTIVE_SCORE_ALCOHOL         => parse_from!(rest, f32, AttractiveScoreAlcohol),
            Self::ATTRACTIVE_SCORE_CULTURE         => parse_from!(rest, f32, AttractiveScoreCulture),
            Self::ATTRACTIVE_SCORE_RELIGION        => parse_from!(rest, f32, AttractiveScoreReligion),
            Self::ATTRACTIVE_SCORE_SPORT           => parse_from!(rest, f32, AttractiveScoreSport),
            Self::ATTRACTIVE_FACTOR_NATURE         => parse_from!(rest, f32, AttractiveFactorNature),
            Self::ATTRACTIVE_FACTOR_NATURE_ADD     => parse_from!(rest, f32, AttractiveFactorNatureAdd),
            Self::ATTRACTIVE_FACTOR_POLLUTION      => parse_from!(rest, f32, AttractiveFactorPollution),
            Self::ATTRACTIVE_FACTOR_POLLUTION_ADD  => parse_from!(rest, f32, AttractiveFactorPollutionAdd),
            Self::ATTRACTIVE_FACTOR_SIGHT          => parse_from!(rest, f32, AttractiveFactorSight),
            Self::ATTRACTIVE_FACTOR_SIGHT_ADD      => parse_from!(rest, f32, AttractiveFactorSightAdd),
            Self::ATTRACTIVE_FACTOR_WATER          => parse_from!(rest, f32, AttractiveFactorWater),
            Self::ATTRACTIVE_FACTOR_WATER_ADD      => parse_from!(rest, f32, AttractiveFactorWaterAdd),

            Self::POLLUTION_HIGH                 => Ok((Self::PollutionHigh,   rest)),
            Self::POLLUTION_MEDIUM               => Ok((Self::PollutionMedium, rest)),
            Self::POLLUTION_SMALL                => Ok((Self::PollutionSmall,  rest)),

            Self::PARTICLE                       => parse_from!(rest, (ParticleType, Point3f, f32, f32), Particle),
            Self::PARTICLE_REACTOR               => parse_from!(rest, Point3f, ParticleReactor),

            Self::TEXT_CAPTION                   => parse_from!(rest, (Point3f, Point3f), TextCaption),
            Self::WORKER_RENDERING_AREA          => parse_from!(rest, (Point3f, Point3f), WorkerRenderingArea),
            Self::RESOURCE_VISUALIZATION         => parse_from!(rest, ResourceVisualization, ResourceVisualization),
            Self::RESOURCE_INCREASE_POINT        => parse_from!(rest, (u32, Point3f), ResourceIncreasePoint),
            Self::RESOURCE_INCREASE_CONV_POINT   => parse_from!(rest, (u32, Point3f, Point3f), ResourceIncreaseConvPoint),
            Self::RESOURCE_FILLING_POINT         => parse_from!(rest, Point3f, ResourceFillingPoint),
            Self::RESOURCE_FILLING_CONV_POINT    => parse_from!(rest, (Point3f, Point3f), ResourceFillingConvPoint),
            Self::WORKING_SFX                    => parse_from!(rest, IdStringParam, WorkingSfx),
            Self::ANIMATION_FPS                  => parse_from!(rest, f32, AnimationFps),
            Self::ANIMATION_MESH                 => parse_from!(rest, (IdStringParam, IdStringParam), AnimationMesh),
            Self::UNDERGROUND_MESH               => parse_from!(rest, (IdStringParam, IdStringParam), UndergroundMesh),


            Self::COST_WORK                      => parse_from!(rest, (ConstructionPhase, f32), CostWork),
            Self::COST_WORK_BUILDING_NODE        => parse_from!(rest, IdStringParam, CostWorkBuildingNode),
            Self::COST_WORK_BUILDING_KEYWORD     => parse_from!(rest, IdStringParam, CostWorkBuildingKeyword),
            Self::COST_WORK_BUILDING_ALL         => Ok((Self::CostWorkBuildingAll, rest)),

            Self::COST_RESOURCE                  => parse_from!(rest, (ResourceType, f32), CostResource),
            Self::COST_RESOURCE_AUTO             => parse_from!(rest, (ConstructionAutoCost, f32), CostResourceAuto),

            Self::COST_WORK_VEHICLE_STATION      => parse_from!(rest, (Point3f, Point3f), CostWorkVehicleStation),
            Self::COST_WORK_VEHICLE_STATION_NODE => parse_from!(rest, IdStringParam, CostWorkVehicleStationNode),

            _ => Err(format!("Unknown token type: \"${}\"", t_type))
        }
    }


    fn parse_connection(src: Option<&'a str>) -> ParseResult<Token<'a>> {
        lazy_static! {
            static ref RX_TYPE: Regex = Regex::new(r"(?s)^([A-Z_]+)(\s*(.*))").unwrap();
        }

        let (con_type, rest) = chop_param(src, &RX_TYPE).map_err(|e| format!("Cannot parse connection type: {}", e))?;

        if let Some(tag) = Connection2PType::from_str(con_type) {
            <(Point3f, Point3f)>::parse(rest).map(|((p1, p2), rest)| (Self::Connection2Points(Tagged2Points { tag, p1, p2 }), rest))
        } else if let Some(tag) = Connection1PType::from_str(con_type) {
            Point3f::parse(rest).map(|(p, rest)| (Self::Connection1Point((tag, p)), rest))
        } else { 
            match con_type {
                Self::CONNECTION_RAIL_DEADEND => Ok((Self::ConnectionRailDeadend, rest)),
                _ => Err(format!("Unknown connection type: {}", con_type))
            }
        }
    }
}



const RX_REMAINDER: &str = r"($|\s*(.*))";

trait ParseSlice<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> where Self: Sized;
}






fn chop_param<'a, 'b>(src: Option<&'a str>, rx: &'b Regex) -> ParseResult<'a, &'a str> {
    let src = src.ok_or(String::from("Chop param failed: not enough data"))?;

    match rx.captures(src) {
        Some(caps) => {
            // allow panic here: this should not happen with valid regex:
            let t = caps.get(1).expect("Regex is broken").as_str();
            let rest = caps.get(3).map(|x| x.as_str());
            Ok((t, rest))
        },
        None => Err(format!("No match in this chunk: [{}]", src))
    }
}

fn parse_param<'a, T, F: Fn(&'a str) -> Result<T, ParseError>>(src: Option<&'a str>, rx: &Regex, f: F) -> ParseResult<'a, T> {
    let (src, rest) = chop_param(src, rx)?;
    let v = f(src).map_err(|e| format!("parse_param failed: {}", e))?;
    Ok((v, rest))
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

impl<'a, T> ParseSlice<'a> for Tagged2Points<T> where T: ParseSlice<'a> {
    fn parse(src: Option<&'a str>) -> ParseResult<Self> {
        let ((tag, p1, p2), src) = <(T, Point3f, Point3f)>::parse(src)?;
        Ok((Tagged2Points { tag, p1, p2 }, src))
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
            Self::TYPE_AIRPLANE_GATE            => Some(Self::AirplaneGate),
            Self::TYPE_AIRPLANE_PARKING         => Some(Self::AirplaneParking),
            Self::TYPE_AIRPLANE_TOWER           => Some(Self::AirplaneTower),
            Self::TYPE_ATTRACTION               => Some(Self::Attraction),
            Self::TYPE_BROADCAST                => Some(Self::Broadcast),
            Self::TYPE_CAR_DEALER               => Some(Self::CarDealer),
            Self::TYPE_CARGO_STATION            => Some(Self::CargoStation),
            Self::TYPE_CHURCH                   => Some(Self::Church),
            Self::TYPE_CITYHALL                 => Some(Self::Cityhall),
            Self::TYPE_CONSTRUCTION_OFFICE      => Some(Self::ConstructionOffice),
            Self::TYPE_CONSTRUCTION_OFFICE_RAIL => Some(Self::ConstructionOfficeRail),
            Self::TYPE_CONTAINER_FACILITY       => Some(Self::ContainerFacility),
            Self::TYPE_COOLING_TOWER            => Some(Self::CoolingTower),
            Self::TYPE_CUSTOMHOUSE              => Some(Self::Customhouse),
            Self::TYPE_DISTRIBUTION_OFFICE      => Some(Self::DistributionOffice),
            Self::TYPE_ELETRIC_EXPORT           => Some(Self::ElectricExport),
            Self::TYPE_ELETRIC_IMPORT           => Some(Self::ElectricImport),
            Self::TYPE_ENGINE                   => Some(Self::Engine),
            Self::TYPE_FACTORY                  => Some(Self::Factory),
            Self::TYPE_FARM                     => Some(Self::Farm),
            Self::TYPE_FIELD                    => Some(Self::Field),
            Self::TYPE_FIRESTATION              => Some(Self::Firestation),
            Self::TYPE_FORKLIFT_GARAGE          => Some(Self::ForkliftGarage),
            Self::TYPE_GARBAGE_OFFICE           => Some(Self::GarbageOffice),
            Self::TYPE_GAS_STATION              => Some(Self::GasStation),
            Self::TYPE_HEATING_ENDSTATION       => Some(Self::HeatingEndstation),
            Self::TYPE_HEATING_PLANT            => Some(Self::HeatingPlant),
            Self::TYPE_HEATING_SWITCH           => Some(Self::HeatingSwitch),
            Self::TYPE_HOSPITAL                 => Some(Self::Hospital),
            Self::TYPE_HOTEL                    => Some(Self::Hotel),
            Self::TYPE_KINDERGARTEN             => Some(Self::Kindergarten),
            Self::TYPE_KINO                     => Some(Self::Kino),
            Self::TYPE_LIVING                   => Some(Self::Living),
            Self::TYPE_MINE_BAUXITE             => Some(Self::MineBauxite),
            Self::TYPE_MINE_COAL                => Some(Self::MineCoal),
            Self::TYPE_MINE_GRAVEL              => Some(Self::MineGravel),
            Self::TYPE_MINE_IRON                => Some(Self::MineIron),
            Self::TYPE_MINE_OIL                 => Some(Self::MineOil),
            Self::TYPE_MINE_URANIUM             => Some(Self::MineUranium),
            Self::TYPE_MINE_WOOD                => Some(Self::MineWood),
            Self::TYPE_MONUMENT                 => Some(Self::Monument),
            Self::TYPE_PARKING                  => Some(Self::Parking),
            Self::TYPE_PEDESTRIAN_BRIDGE        => Some(Self::PedestrianBridge),
            Self::TYPE_POLICE_STATION           => Some(Self::PoliceStation),
            Self::TYPE_PASSANGER_STATION        => Some(Self::PassangerStation),
            Self::TYPE_POLLUTION_METER          => Some(Self::PollutionMeter),
            Self::TYPE_POWERPLANT               => Some(Self::Powerplant),
            Self::TYPE_PRODUCTION_LINE          => Some(Self::ProductionLine),
            Self::TYPE_PUB                      => Some(Self::Pub),
            Self::TYPE_RAIL_TRAFO               => Some(Self::RailTrafo),
            Self::TYPE_RAILDEPO                 => Some(Self::Raildepo),
            Self::TYPE_ROADDEPO                 => Some(Self::Roaddepo),
            Self::TYPE_SCHOOL                   => Some(Self::School),
            Self::TYPE_SHIP_DOCK                => Some(Self::ShipDock),
            Self::TYPE_SHOP                     => Some(Self::Shop),
            Self::TYPE_SPORT                    => Some(Self::Sport),
            Self::TYPE_STORAGE                  => Some(Self::Storage),
            Self::TYPE_SUBSTATION               => Some(Self::Substation),
            Self::TYPE_TRANSFORMATOR            => Some(Self::Transformator),
            Self::TYPE_UNIVERSITY               => Some(Self::University),
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


impl BuildingSubtype {
    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::SUBTYPE_AIRCUSTOM          => Some(Self::Aircustom),
            Self::SUBTYPE_AIRPLANE           => Some(Self::Airplane),
            Self::SUBTYPE_CABLEWAY           => Some(Self::Cableway),
            Self::SUBTYPE_HOSTEL             => Some(Self::Hostel),
            Self::SUBTYPE_MEDICAL            => Some(Self::Medical),
            Self::SUBTYPE_RADIO              => Some(Self::Radio),
            Self::SUBTYPE_RAIL               => Some(Self::Rail),
            Self::SUBTYPE_RESTAURANT         => Some(Self::Restaurant),
            Self::SUBTYPE_ROAD               => Some(Self::Road),
            Self::SUBTYPE_SHIP               => Some(Self::Ship),
            Self::SUBTYPE_SOVIET             => Some(Self::Soviet),
            Self::SUBTYPE_SPACE_FOR_VEHICLES => Some(Self::SpaceForVehicles),
            Self::SUBTYPE_TECHNICAL          => Some(Self::Technical),
            Self::SUBTYPE_TELEVISION         => Some(Self::Television),
            Self::SUBTYPE_TROLLEYBUS         => Some(Self::Trolleybus),
            _ => None
        }
    }
}


impl ParseSlice<'_> for BuildingSubtype {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Self::from_str(s).ok_or(format!("Unknown building subtype '{}'", s)))
    }
}


impl Connection2PType {
    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::CONN_AIRROAD        => Some(Self::AirRoad),
            Self::CONN_PED            => Some(Self::Pedestrian),
            Self::CONN_PED_NOTPICK    => Some(Self::PedestrianNotPick),
            Self::CONN_ROAD           => Some(Self::Road),
            Self::CONN_ROAD_ALLOWPASS => Some(Self::RoadAllowpass),
            Self::CONN_ROAD_BORDER    => Some(Self::RoadBorder),
            Self::CONN_ROAD_IN        => Some(Self::RoadIn),
            Self::CONN_ROAD_OUT       => Some(Self::RoadOut),
            Self::CONN_RAIL           => Some(Self::Rail),
            Self::CONN_RAIL_ALLOWPASS => Some(Self::RailAllowpass),
            Self::CONN_RAIL_BORDER    => Some(Self::RailBorder),
            Self::CONN_RAIL_HEIGHT    => Some(Self::RailHeight),
            Self::CONN_HEATING_BIG    => Some(Self::HeatingBig),
            Self::CONN_HEATING_SMALL  => Some(Self::HeatingSmall),
            Self::CONN_STEAM_IN       => Some(Self::SteamIn),
            Self::CONN_STEAM_OUT      => Some(Self::SteamOut),
            Self::CONN_PIPE_IN        => Some(Self::PipeIn),
            Self::CONN_PIPE_OUT       => Some(Self::PipeOut),
            Self::CONN_BULK_IN        => Some(Self::BulkIn),
            Self::CONN_BULK_OUT       => Some(Self::BulkOut),
            Self::CONN_CABLEWAY       => Some(Self::Cableway),
            Self::CONN_FACTORY        => Some(Self::Factory),
            Self::CONN_CONVEYOR_IN    => Some(Self::ConveyorIn),
            Self::CONN_CONVEYOR_OUT   => Some(Self::ConveyorOut),
            Self::CONN_ELECTRIC_H_IN  => Some(Self::ElectricHighIn),
            Self::CONN_ELECTRIC_H_OUT => Some(Self::ElectricHighOut),
            Self::CONN_ELECTRIC_L_IN  => Some(Self::ElectricLowIn),
            Self::CONN_ELECTRIC_L_OUT => Some(Self::ElectricLowOut),
            Self::CONN_FENCE          => Some(Self::Fence),
            _ => None
        }
    }
}


impl ParseSlice<'_> for Connection2PType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Self::from_str(s).ok_or(format!("Unknown 2-point connection type '{}'", s)))
    }
}


impl Connection1PType {
    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::ROAD_DEAD       => Some(Self::RoadDead),
            Self::PEDESTRIAN_DEAD => Some(Self::PedestrianDead),
            Self::WATER_DEAD      => Some(Self::WaterDead),
            Self::AIRPORT_DEAD    => Some(Self::AirportDead),
            Self::ADVANCED_POINT  => Some(Self::AdvancedPoint),
            _ => None
        }
    }
}

impl ParseSlice<'_> for Connection1PType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| Connection1PType::from_str(s).ok_or(format!("Unknown 1-point connection type '{}'", s)))
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
            Self::VEHICLES  => Some(Self::Vehicles),
            Self::NUCLEAR1  => Some(Self::Nuclear1),
            Self::NUCLEAR2  => Some(Self::Nuclear2),
            _ => None
        }
    }
}

impl ParseSlice<'_> for StorageCargoType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([0-9A-Z_]+)", RX_REMAINDER)).unwrap();
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
            Self::FOUNTAIN_1           => Some(Self::Fountain1),
            Self::FOUNTAIN_2           => Some(Self::Fountain2),
            Self::FOUNTAIN_3           => Some(Self::Fountain3),
            _ => None
        }
    }
}

impl ParseSlice<'_> for ParticleType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([0-9a-z_]+)", RX_REMAINDER)).unwrap();
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
            Self::ALCOHOL       => Some(Self::Alcohol),
            Self::ALUMINA       => Some(Self::Alumina),
            Self::ALUMINIUM     => Some(Self::Aluminium),
            Self::ASPHALT       => Some(Self::Asphalt),
            Self::BAUXITE       => Some(Self::Bauxite),
            Self::BITUMEN       => Some(Self::Bitumen),
            Self::BOARDS        => Some(Self::Boards),
            Self::BRICKS        => Some(Self::Bricks),
            Self::CEMENT        => Some(Self::Cement),
            Self::CHEMICALS     => Some(Self::Chemicals),
            Self::CLOTHES       => Some(Self::Clothes),
            Self::COAL          => Some(Self::Coal),
            Self::CONCRETE      => Some(Self::Concrete),
            Self::CROPS         => Some(Self::Crops),
            Self::ELECTRO_COMP  => Some(Self::ElectroComponents),
            Self::ELECTRICITY   => Some(Self::Electricity),
            Self::ELECTRONICS   => Some(Self::Electronics),
            Self::FABRIC        => Some(Self::Fabric),
            Self::FOOD          => Some(Self::Food),
            Self::FUEL          => Some(Self::Fuel),
            Self::GRAVEL        => Some(Self::Gravel),
            Self::HEAT          => Some(Self::Heat),
            Self::IRON          => Some(Self::Iron),
            Self::LIVESTOCK     => Some(Self::Livestock),
            Self::MECH_COMP     => Some(Self::MechComponents),
            Self::MEAT          => Some(Self::Meat),
            Self::NUCLEAR_FUEL  => Some(Self::NuclearFuel),
            Self::NUCLEAR_WASTE => Some(Self::NuclearWaste),
            Self::OIL           => Some(Self::Oil),
            Self::PLASTIC       => Some(Self::Plastic),
            Self::PREFABS       => Some(Self::PrefabPanels),
            Self::RAW_BAUXITE   => Some(Self::RawBauxite),
            Self::RAW_COAL      => Some(Self::RawCoal),
            Self::RAW_GRAVEL    => Some(Self::RawGravel),
            Self::RAW_IRON      => Some(Self::RawIron),
            Self::STEEL         => Some(Self::Steel),
            Self::UF_6          => Some(Self::UF6),
            Self::URANIUM       => Some(Self::Uranium),
            Self::VEHICLES      => Some(Self::Vehicles),
            Self::WOOD          => Some(Self::Wood),
            Self::WORKERS       => Some(Self::Workers),
            Self::YELLOWCAKE    => Some(Self::Yellowcake),
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


impl AirplaneStationType {
    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::AIRPLANE_STATION_30M => Some(Self::M30),
            Self::AIRPLANE_STATION_40M => Some(Self::M40),
            Self::AIRPLANE_STATION_50M => Some(Self::M50),
            Self::AIRPLANE_STATION_75M => Some(Self::M75),
            _ => None
        }
    }
}

impl ParseSlice<'_> for AirplaneStationType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([0-9]+M)", RX_REMAINDER)).unwrap();
        }

        parse_param(src, &RX, |s| AirplaneStationType::from_str(s).ok_or(format!("Unknown airplane station type '{}'", s)))
    }
}


impl AttractionType {
    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::ATTRACTION_TYPE_CARUSEL => Some(Self::Carousel),
            Self::ATTRACTION_TYPE_GALLERY => Some(Self::Gallery),
            Self::ATTRACTION_TYPE_MUSEUM  => Some(Self::Museum),
            Self::ATTRACTION_TYPE_SIGHT   => Some(Self::Sight),
            Self::ATTRACTION_TYPE_SWIM    => Some(Self::Swim),
            Self::ATTRACTION_TYPE_ZOO     => Some(Self::Zoo),
            _ => None
        }
    }
}

impl ParseSlice<'_> for AttractionType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }
        
        parse_param(src, &RX, |s| AttractionType::from_str(s).ok_or(format!("Unknown attraction type '{}'", s)))
    }
}


impl ResourceSourceType {
    fn from_str(src: &str) -> Option<Self> {
        match src {
            Self::RES_SOURCE_ASPHALT         => Some(Self::Asphalt),
            Self::RES_SOURCE_CONCRETE        => Some(Self::Concrete),
            Self::RES_SOURCE_COVERED         => Some(Self::Covered),
            Self::RES_SOURCE_COVERED_ELECTRO => Some(Self::CoveredElectro),
            Self::RES_SOURCE_GRAVEL          => Some(Self::Gravel),
            Self::RES_SOURCE_OPEN            => Some(Self::Open),
            Self::RES_SOURCE_OPEN_BOARDS     => Some(Self::OpenBoards),
            Self::RES_SOURCE_OPEN_BRICKS     => Some(Self::OpenBricks),
            Self::RES_SOURCE_OPEN_PANELS     => Some(Self::OpenPanels),
            Self::RES_SOURCE_WORKERS         => Some(Self::Workers),

            _ => None
        }
    }
}

impl ParseSlice<'_> for ResourceSourceType {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX: Regex = Regex::new(concatcp!(r"(?s)^([A-Z_]+)", RX_REMAINDER)).unwrap();
        }
        
        parse_param(src, &RX, |s| ResourceSourceType::from_str(s).ok_or(format!("Unknown resource-source type '{}'", s)))
    }
}


impl ParseSlice<'_> for ResourceVisualization {
    fn parse(src: Option<&str>) -> ParseResult<Self> {
        lazy_static! {
            static ref RX_ALL: Regex = Regex::new(concatcp!(r"(?s)^([a-z]+)", RX_REMAINDER)).unwrap();
            // these seem to be just for information. Only correct number-order matters
            //static ref RX_1: Regex = Regex::new(concatcp!(r"(?s)^(position)", RX_REMAINDER)).unwrap();
            //static ref RX_2: Regex = Regex::new(concatcp!(r"(?s)^(rotation)", RX_REMAINDER)).unwrap();
            //static ref RX_3: Regex = Regex::new(concatcp!(r"(?s)^(scale)",    RX_REMAINDER)).unwrap();
            //static ref RX_4: Regex = Regex::new(concatcp!(r"(?s)^(numstepx)", RX_REMAINDER)).unwrap();
            //static ref RX_5: Regex = Regex::new(concatcp!(r"(?s)^(numstept)", RX_REMAINDER)).unwrap();
        }
        
        let (storage_id, src) = u32::parse(src)?;
        let (_, src)         = chop_param(src, &RX_ALL)?;
        let (position, src)  = Point3f::parse(src)?;
        let (_, src)         = chop_param(src, &RX_ALL)?;
        let (rotation, src)  = f32::parse(src)?;
        let (_, src)         = chop_param(src, &RX_ALL)?;
        let (scale, src)     = Point3f::parse(src)?;
        let (_, src)         = chop_param(src, &RX_ALL)?;
        let (numstep_x, src) = <(f32, u32)>::parse(src)?;
        let (_, src)        = chop_param(src, &RX_ALL)?;
        let (numstep_z, src) = <(f32, u32)>::parse(src)?;

        Ok((ResourceVisualization { storage_id, position, rotation, scale, numstep_x, numstep_z }, src))
    }
}




lazy_static! {
    //static ref RX_SPLIT: Regex = Regex::new(concatcp!("(^|", r"(((\s*\r?)|((--|//)[^\n]*))\n)+", r")(\$|end\s*(\r?\n\s*)*)")).unwrap();
    static ref RX_SPLIT: Regex = Regex::new(concatcp!("(^|", r"(\s*((--|//)[^\n]*)?\r?\n)+", r")(\$|end\s*(\r?\n\s*)*)")).unwrap();
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
