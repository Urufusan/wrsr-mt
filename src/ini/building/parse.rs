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

            Connection2PType,
            Connection1PType,
            AirplaneStationType,
            AttractionType,
            ResourceSourceType,
           };

use crate::ini::common::{ParseSlice, 
                         ParseResult, 
                         ParseError, 
                         Point3f,
                         Rect,
                         QuotedStringParam,
                         IdStringParam,
                         CostKeywordParam,
                         RX_REMAINDER, 
                         chop_param, 
                         parse_param,
                         parse_tokens_with,
                         parse_tokens_strict_with,
                        };



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
        macro_rules! parse {
            ($id:ident, $t:ty) => {
                <$t>::parse(rest).map(|(p, rest)| (Self::$id(p), rest))
            };
            ($id:ident) => {
                Ok((Self::$id, rest))
            };
        }

        match t_type {
            Self::NAME_STR                       => parse!(NameStr,                     QuotedStringParam),
            Self::NAME                           => parse!(Name,                        u32),

            Self::BUILDING_TYPE                  => parse!(BuildingType,                BuildingType),
            Self::BUILDING_SUBTYPE               => parse!(BuildingSubtype,             BuildingSubtype),

            Self::HEATING_ENABLE                 => parse!(HeatEnable),
            Self::HEATING_DISABLE                => parse!(HeatDisable),
            Self::CIVIL_BUILDING                 => parse!(CivilBuilding),
            Self::MONUMENT_TRESPASS              => parse!(MonumentTrespass),
            Self::QUALITY_OF_LIVING              => parse!(QualityOfLiving,             f32),

            Self::WORKERS_NEEDED                 => parse!(WorkersNeeded,               u32),
            Self::PROFESSORS_NEEDED              => parse!(ProfessorsNeeded,            u32),
            Self::CITIZEN_ABLE_SERVE             => parse!(CitizenAbleServe,            u32),
            Self::CONSUMPTION                    => parse!(Consumption,                 (ResourceType, f32)),
            Self::CONSUMPTION_PER_SEC            => parse!(ConsumptionPerSec,           (ResourceType, f32)),
            Self::PRODUCTION                     => parse!(Production,                  (ResourceType, f32)),
            Self::PRODUCTION_SUN                 => parse!(ProductionSun,               f32),
            Self::PRODUCTION_WIND                => parse!(ProductionWind,              f32),
            Self::SEASONAL_TEMP_MIN              => parse!(SeasonalTempMin,             f32),
            Self::SEASONAL_TEMP_MAX              => parse!(SeasonalTempMax,             f32),

            Self::ELE_CONSUM_WORKER_FACTOR_BASE  => parse!(EleConsumWorkerFactorBase,   f32),
            Self::ELE_CONSUM_WORKER_FACTOR_NIGHT => parse!(EleConsumWorkerFactorNight,  f32),
            Self::ELE_CONSUM_SERVE_FACTOR_BASE   => parse!(EleConsumServeFactorBase,    f32),
            Self::ELE_CONSUM_SERVE_FACTOR_NIGHT  => parse!(EleConsumServeFactorNight,   f32),
            Self::ELE_CONSUM_CARGO_LOAD_FACTOR   => parse!(EleConsumCargoLoadFactor,    f32),
            Self::ELE_CONSUM_CARGO_UNLOAD_FACTOR => parse!(EleConsumCargoUnloadFactor,  f32),

            Self::NO_ELE_WORK_FACTOR_BASE        => parse!(NoEleWorkFactorBase,         f32),
            Self::NO_ELE_WORK_FACTOR_NIGHT       => parse!(NoEleWorkFactorNight,        f32),
            Self::NO_HEAT_WORK_FACTOR            => parse!(NoHeatWorkFactor,            f32),

            Self::ENGINE_SPEED                   => parse!(EngineSpeed,                 f32),
            Self::CABLEWAY_HEAVY                 => parse!(CablewayHeavy),
            Self::CABLEWAY_LIGHT                 => parse!(CablewayLight),
            Self::RESOURCE_SOURCE                => parse!(ResourceSource,              ResourceSourceType),

            Self::STORAGE                        => parse!(Storage,                     (StorageCargoType, f32)),
            Self::STORAGE_SPECIAL                => parse!(StorageSpecial,              (StorageCargoType, f32, ResourceType)),
            Self::STORAGE_FUEL                   => parse!(StorageFuel,                 (StorageCargoType, f32)),
            Self::STORAGE_EXPORT                 => parse!(StorageExport,               (StorageCargoType, f32)),
            Self::STORAGE_IMPORT                 => parse!(StorageImport,               (StorageCargoType, f32)),
            Self::STORAGE_IMPORT_CARPLANT        => parse!(StorageImportCarplant,       (StorageCargoType, f32)),
            Self::STORAGE_EXPORT_SPECIAL         => parse!(StorageExportSpecial,        (StorageCargoType, f32, ResourceType)),
            Self::STORAGE_IMPORT_SPECIAL         => parse!(StorageImportSpecial,        (StorageCargoType, f32, ResourceType)),
            Self::STORAGE_DEMAND_BASIC           => parse!(StorageDemandBasic,          (StorageCargoType, f32)),
            Self::STORAGE_DEMAND_MEDIUMADVANCED  => parse!(StorageDemandMediumAdvanced, (StorageCargoType, f32)),
            Self::STORAGE_DEMAND_ADVANCED        => parse!(StorageDemandAdvanced,       (StorageCargoType, f32)),
            Self::STORAGE_DEMAND_HOTEL           => parse!(StorageDemandHotel,          (StorageCargoType, f32)),
            Self::STORAGE_PACK_FROM              => parse!(StoragePackFrom,             u32),
            Self::STORAGE_UNPACK_TO              => parse!(StorageUnpackTo,             u32),
            Self::STORAGE_LIVING_AUTO            => parse!(StorageLivingAuto,           IdStringParam),

            Self::VEHICLE_LOADING_FACTOR         => parse!(VehicleLoadingFactor,        f32),
            Self::VEHICLE_UNLOADING_FACTOR       => parse!(VehicleUnloadingFactor,      f32),
            
            Self::ROAD_VEHICLE_NOT_FLIP          => parse!(RoadNotFlip),
            Self::ROAD_VEHICLE_ELECTRIC          => parse!(RoadElectric),
            Self::VEHICLE_CANNOT_SELECT          => parse!(VehicleCannotSelect),
            Self::LONG_TRAINS                    => parse!(LongTrains),

            Self::WORKING_VEHICLES_NEEDED        => parse!(WorkingVehiclesNeeded,       u32),
            Self::VEHICLE_STATION                => parse!(VehicleStation,              (Point3f, Point3f)),
            Self::VEHICLE_STATION_NOT_BLOCK      => parse!(VehicleStationNotBlock),
            Self::VEHICLE_STATION_DETOUR_POINT   => parse!(VehicleStationDetourPoint,   Point3f),
            Self::VEHICLE_STATION_DETOUR_PID     => parse!(VehicleStationDetourPid,     (u32, Point3f)),

            Self::VEHICLE_PARKING                => parse!(VehicleParking,              (Point3f, Point3f)),
            Self::VEHICLE_PARKING_DETOUR_POINT   => parse!(VehicleParkingDetourPoint,   Point3f),
            Self::VEHICLE_PARKING_DETOUR_PID     => parse!(VehicleParkingDetourPid,     (u32, Point3f)),
            Self::VEHICLE_PARKING_PERSONAL       => parse!(VehicleParkingPersonal,      (Point3f, Point3f)),

            Self::AIRPLANE_STATION               => parse!(AirplaneStation,             (AirplaneStationType, Point3f, Point3f)),
            Self::HELIPORT_STATION               => parse!(HeliportStation,             (Point3f, Point3f)),
            Self::SHIP_STATION                   => parse!(ShipStation,                 (Point3f, Point3f)),
            Self::HELIPORT_AREA                  => parse!(HeliportArea,                f32),
            Self::HARBOR_OVER_TERRAIN_FROM       => parse!(HarborTerrainFrom,           f32),
            Self::HARBOR_OVER_WATER_FROM         => parse!(HarborWaterFrom,             f32),
            Self::HARBOR_EXTEND_WHEN_BULDING     => parse!(HarborExtendWhenBuilding,    f32),

            Self::CONNECTION => Self::parse_connection(rest),

            Self::CONNECTIONS_SPACE                => parse!(ConnectionsSpace,             Rect),
            Self::CONNECTIONS_ROAD_DEAD_SQUARE     => parse!(ConnectionsRoadDeadSquare,    Rect),
            Self::CONNECTIONS_AIRPORT_DEAD_SQUARE  => parse!(ConnectionsAirportDeadSquare, Rect),
            Self::CONNECTIONS_WATER_DEAD_SQUARE    => parse!(ConnectionsWaterDeadSquare,   (f32, Rect)),
            Self::OFFSET_CONNECTION_XYZW           => parse!(OffsetConnection,             (u32, Point3f)),

            Self::ATTRACTION_TYPE                  => parse!(AttractionType,               (AttractionType, u32)),
            Self::ATTRACTION_REMEMBER_USAGE        => parse!(AttractionRememberUsage),
            Self::ATTRACTIVE_SCORE_BASE            => parse!(AttractiveScoreBase,          f32),
            Self::ATTRACTIVE_SCORE_ALCOHOL         => parse!(AttractiveScoreAlcohol,       f32),
            Self::ATTRACTIVE_SCORE_CULTURE         => parse!(AttractiveScoreCulture,       f32),
            Self::ATTRACTIVE_SCORE_RELIGION        => parse!(AttractiveScoreReligion,      f32),
            Self::ATTRACTIVE_SCORE_SPORT           => parse!(AttractiveScoreSport,         f32),
            Self::ATTRACTIVE_FACTOR_NATURE         => parse!(AttractiveFactorNature,       f32),
            Self::ATTRACTIVE_FACTOR_NATURE_ADD     => parse!(AttractiveFactorNatureAdd,    f32),
            Self::ATTRACTIVE_FACTOR_POLLUTION      => parse!(AttractiveFactorPollution,    f32),
            Self::ATTRACTIVE_FACTOR_POLLUTION_ADD  => parse!(AttractiveFactorPollutionAdd, f32),
            Self::ATTRACTIVE_FACTOR_SIGHT          => parse!(AttractiveFactorSight,        f32),
            Self::ATTRACTIVE_FACTOR_SIGHT_ADD      => parse!(AttractiveFactorSightAdd,     f32),
            Self::ATTRACTIVE_FACTOR_WATER          => parse!(AttractiveFactorWater,        f32),
            Self::ATTRACTIVE_FACTOR_WATER_ADD      => parse!(AttractiveFactorWaterAdd,     f32),

            Self::POLLUTION_HIGH                   => parse!(PollutionHigh),
            Self::POLLUTION_MEDIUM                 => parse!(PollutionMedium),
            Self::POLLUTION_SMALL                  => parse!(PollutionSmall),

            Self::PARTICLE                         => parse!(Particle,                    (ParticleType, Point3f, f32, f32)),
            Self::PARTICLE_REACTOR                 => parse!(ParticleReactor,             Point3f),
            Self::PARTICLE_SNOW_REMOVE             => parse!(ParticleSnowRemove,          (Point3f, u32, f32)),

            Self::TEXT_CAPTION                     => parse!(TextCaption,                 (Point3f, Point3f)),
            Self::WORKER_RENDERING_AREA            => parse!(WorkerRenderingArea,         (Point3f, Point3f)),
            Self::RESOURCE_VISUALIZATION           => parse!(ResourceVisualization,       ResourceVisualization),
            Self::RESOURCE_INCREASE_POINT          => parse!(ResourceIncreasePoint,       (u32, Point3f)),
            Self::RESOURCE_INCREASE_CONV_POINT     => parse!(ResourceIncreaseConvPoint,   (u32, Point3f, Point3f)),
            Self::RESOURCE_FILLING_POINT           => parse!(ResourceFillingPoint,        Point3f),
            Self::RESOURCE_FILLING_CONV_POINT      => parse!(ResourceFillingConvPoint,    (Point3f, Point3f)),
            Self::WORKING_SFX                      => parse!(WorkingSfx,                  IdStringParam),
            Self::ANIMATION_FPS                    => parse!(AnimationFps,                f32),
            Self::ANIMATION_MESH                   => parse!(AnimationMesh,               (IdStringParam, IdStringParam)),
            Self::UNDERGROUND_MESH                 => parse!(UndergroundMesh,             (IdStringParam, IdStringParam)),

            Self::COST_WORK                        => parse!(CostWork,                    (ConstructionPhase, f32)),
            Self::COST_WORK_BUILDING_NODE          => parse!(CostWorkBuildingNode,        IdStringParam),
            Self::COST_WORK_BUILDING_KEYWORD       => parse!(CostWorkBuildingKeyword,     CostKeywordParam),
            Self::COST_WORK_BUILDING_ALL           => parse!(CostWorkBuildingAll),

            Self::COST_RESOURCE                    => parse!(CostResource,                (ResourceType, f32)),
            Self::COST_RESOURCE_AUTO               => parse!(CostResourceAuto,            (ConstructionAutoCost, f32)),

            Self::COST_WORK_VEHICLE_STATION        => parse!(CostWorkVehicleStation,      (Point3f, Point3f)),
            Self::COST_WORK_VEHICLE_STATION_NODE   => parse!(CostWorkVehicleStationNode,  IdStringParam),

            _ => Err(format!("Unknown token type: \"${}\"", t_type))
        }
    }


    fn parse_connection(src: Option<&'a str>) -> ParseResult<Token<'a>> {
        lazy_static! {
            static ref RX_TYPE: Regex = Regex::new(r"(?s)^([A-Z_]+)(\s*(.*))").unwrap();
        }

        let (con_type, rest) = chop_param(src, &RX_TYPE).map_err(|e| format!("Cannot parse connection type: {}", e))?;

        if let Some(tag) = Connection2PType::from_str(con_type) {
            <(Point3f, Point3f)>::parse(rest).map(|((p1, p2), rest)| (Self::Connection2Points((tag, p1, p2)), rest))
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
            Self::ASPHALT_LAYING   => Some(Self::AsphaltLaying),
            Self::ASPHALT_ROLLING  => Some(Self::AsphaltRolling),
            Self::BOARDS_LAYING    => Some(Self::BoardsLaying),
            Self::BRICKS_LAYING    => Some(Self::BricksLaying),
            Self::BRIDGE_BUILDING  => Some(Self::BridgeBuilding),
            Self::GRAVEL_LAYING    => Some(Self::GravelLaying),
            Self::GROUNDWORKS      => Some(Self::Groundworks),
            Self::INTERIOR_WORKS   => Some(Self::InteriorWorks),
            Self::PANELS_LAYING    => Some(Self::PanelsLaying),
            Self::RAILWAY_LAYING   => Some(Self::RailwayLaying),
            Self::ROOFTOP_BUILDING => Some(Self::RooftopBuilding),
            Self::SKELETON_CASTING => Some(Self::SkeletonCasting),
            Self::STEEL_LAYING     => Some(Self::SteelLaying),
            Self::TUNNELING        => Some(Self::Tunneling),
            Self::WIRE_LAYING      => Some(Self::WireLaying),
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
            Self::GROUND             => Some(Self::Ground),
            Self::GROUND_ASPHALT     => Some(Self::GroundAsphalt),
            Self::WALL_CONCRETE      => Some(Self::WallConcrete),
            Self::WALL_PANELS        => Some(Self::WallPanels),
            Self::WALL_BRICK         => Some(Self::WallBrick),
            Self::WALL_STEEL         => Some(Self::WallSteel),
            Self::WALL_WOOD          => Some(Self::WallWood),
            Self::TECH_STEEL         => Some(Self::TechSteel),
            Self::ELECTRO_STEEL      => Some(Self::ElectroSteel),
            Self::TECH_ELECTRO_STEEL => Some(Self::TechElectroSteel),
            Self::ROOF_WOOD_BRICK    => Some(Self::RoofWoodBrick),
            Self::ROOF_STEEL         => Some(Self::RoofSteel),
            Self::ROOF_WOOD_STEEL    => Some(Self::RoofWoodSteel),
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
    static ref RX_SPLIT: Regex = Regex::new(concatcp!("(^|", r"(\s*((--|//)[^\n]*)?\r?\n)+", r")(\$|end\s*(\r?\n\s*)*)")).unwrap();
}


#[inline]
pub fn parse_tokens<'s>(src: &'s str) -> Vec<(&'s str, ParseResult<'s, Token<'s>>)> {
    parse_tokens_with(src, &RX_SPLIT, Token::parse)
}


#[inline]
pub fn parse_tokens_strict<'a>(src: &'a str) -> Result<Vec<(&'a str, Token<'a>)>, Vec<(&'a str, ParseError)>> {
    parse_tokens_strict_with(src, &RX_SPLIT, Token::parse)
}
