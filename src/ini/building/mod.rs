mod display;
mod parse;

use crate::ini::common::{Point3f, Rect, QuotedStringParam, IdStringParam, CostKeywordParam};

pub use parse::{parse_tokens, parse_tokens_strict};

//#[derive(Clone)]
pub enum Token<'a> {

    NameStr(QuotedStringParam<'a>),
    Name(u32),

    BuildingType(BuildingType),
    BuildingSubtype(BuildingSubtype),
    HeatEnable,
    HeatDisable,

    CivilBuilding,
    MonumentTrespass,
    QualityOfLiving(f32),

    WorkersNeeded(u32),
    ProfessorsNeeded(u32),
    CitizenAbleServe(u32),
    Consumption((ResourceType, f32)),
    ConsumptionPerSec((ResourceType, f32)),
    Production((ResourceType, f32)),
    ProductionSun(f32),
    ProductionWind(f32),
    SeasonalTempMin(f32),
    SeasonalTempMax(f32),

    EleConsumWorkerFactorNight(f32),
    EleConsumWorkerFactorBase(f32),
    EleConsumServeFactorNight(f32),
    EleConsumServeFactorBase(f32),
    EleConsumCargoLoadFactor(f32),
    EleConsumCargoUnloadFactor(f32),

    NoEleWorkFactorBase(f32),
    NoEleWorkFactorNight(f32),
    NoHeatWorkFactor(f32),

    EngineSpeed(f32),
    CablewayHeavy,
    CablewayLight,
    ResourceSource(ResourceSourceType),

    Storage((StorageCargoType, f32)),
    StorageSpecial((StorageCargoType, f32, ResourceType)),
    StorageFuel((StorageCargoType, f32)),
    StorageExport((StorageCargoType, f32)),
    StorageImport((StorageCargoType, f32)),
    StorageImportCarplant((StorageCargoType, f32)),
    StorageExportSpecial((StorageCargoType, f32, ResourceType)),
    StorageImportSpecial((StorageCargoType, f32, ResourceType)),
    StorageDemandBasic((StorageCargoType, f32)),
    StorageDemandMediumAdvanced((StorageCargoType, f32)),
    StorageDemandAdvanced((StorageCargoType, f32)),
    StorageDemandHotel((StorageCargoType, f32)),
    StoragePackFrom(u32),
    StorageUnpackTo(u32),
    StorageLivingAuto(IdStringParam<'a>),

    VehicleLoadingFactor(f32),
    VehicleUnloadingFactor(f32),

    RoadNotFlip,
    RoadElectric,
    VehicleCannotSelect,
    LongTrains,

    WorkingVehiclesNeeded(u32),
    VehicleStation((Point3f, Point3f)),
    VehicleStationNotBlock,
    VehicleStationDetourPoint(Point3f),
    VehicleStationDetourPid((u32, Point3f)),

    VehicleParking((Point3f, Point3f)),
    VehicleParkingDetourPoint(Point3f),
    VehicleParkingDetourPid((u32, Point3f)),
    VehicleParkingPersonal((Point3f, Point3f)),

    AirplaneStation((AirplaneStationType, Point3f, Point3f)),
    HeliportStation((Point3f, Point3f)),
    ShipStation((Point3f, Point3f)),
    HeliportArea(f32),
    HarborTerrainFrom(f32),
    HarborWaterFrom(f32),
    HarborExtendWhenBuilding(f32),

    Connection2Points((Connection2PType, Point3f, Point3f)),
    Connection1Point((Connection1PType, Point3f)),
    OffsetConnection((u32, Point3f)),
    ConnectionRailDeadend,

    ConnectionsSpace(Rect),
    ConnectionsRoadDeadSquare(Rect),
    ConnectionsAirportDeadSquare(Rect),
    ConnectionsWaterDeadSquare((f32, Rect)),

    AttractionType((AttractionType, u32)),
    AttractionRememberUsage,
    AttractiveScoreBase(f32),
    AttractiveScoreAlcohol(f32),
    AttractiveScoreCulture(f32),
    AttractiveScoreReligion(f32),
    AttractiveScoreSport(f32),
    AttractiveFactorNature(f32),
    AttractiveFactorNatureAdd(f32),
    AttractiveFactorPollution(f32),
    AttractiveFactorPollutionAdd(f32),
    AttractiveFactorSight(f32),
    AttractiveFactorSightAdd(f32),
    AttractiveFactorWater(f32),
    AttractiveFactorWaterAdd(f32),

    PollutionHigh,
    PollutionMedium,
    PollutionSmall,

    Particle((ParticleType, Point3f, f32, f32)),
    ParticleReactor(Point3f),
    TextCaption((Point3f, Point3f)),
    WorkerRenderingArea((Point3f, Point3f)),
    ResourceVisualization(ResourceVisualization),
    ResourceIncreasePoint((u32, Point3f)),
    ResourceIncreaseConvPoint((u32, Point3f, Point3f)),
    ResourceFillingPoint(Point3f),
    ResourceFillingConvPoint((Point3f, Point3f)),
    WorkingSfx(IdStringParam<'a>),
    AnimationFps(f32),
    AnimationMesh((IdStringParam<'a>, IdStringParam<'a>)),
    UndergroundMesh((IdStringParam<'a>, IdStringParam<'a>)),

    CostWork((ConstructionPhase, f32)),
    CostWorkBuildingNode(IdStringParam<'a>),
    CostWorkBuildingKeyword(CostKeywordParam<'a>),
    CostWorkBuildingAll,

    CostResource((ResourceType, f32)),
    CostResourceAuto((ConstructionAutoCost, f32)),

    CostWorkVehicleStation((Point3f, Point3f)),
    CostWorkVehicleStationNode(IdStringParam<'a>),
}


impl<'a> Token<'a> {
    const NAME_STR:                       &'static str = "NAME_STR";
    const NAME:                           &'static str = "NAME";
    const BUILDING_TYPE:                  &'static str = "TYPE_";
    const BUILDING_SUBTYPE:               &'static str = "SUBTYPE_";

    const HEATING_ENABLE:                 &'static str = "HEATING_ENABLE";
    const HEATING_DISABLE:                &'static str = "HEATING_DISABLE";
    const CIVIL_BUILDING:                 &'static str = "CIVIL_BUILDING";
    const MONUMENT_TRESPASS:              &'static str = "MONUMENT_ENABLE_TRESPASSING";
    const QUALITY_OF_LIVING:              &'static str = "QUALITY_OF_LIVING";

    const WORKERS_NEEDED:                 &'static str = "WORKERS_NEEDED";
    const PROFESSORS_NEEDED:              &'static str = "PROFESORS_NEEDED";
    const CITIZEN_ABLE_SERVE:             &'static str = "CITIZEN_ABLE_SERVE";
    const CONSUMPTION:                    &'static str = "CONSUMPTION";
    const CONSUMPTION_PER_SEC:            &'static str = "CONSUMPTION_PER_SECOND";
    const PRODUCTION:                     &'static str = "PRODUCTION";
    const PRODUCTION_SUN:                 &'static str = "PRODUCTION_CONNECT_TO_SUN";
    const PRODUCTION_WIND:                &'static str = "PRODUCTION_CONNECT_TO_WIND";
    const SEASONAL_TEMP_MIN:              &'static str = "SEASONAL_CLOSE_IF_TEMP_BELLOW";
    const SEASONAL_TEMP_MAX:              &'static str = "SEASONAL_CLOSE_IF_TEMP_ABOVE";

    const ELE_CONSUM_WORKER_FACTOR_BASE:  &'static str = "ELETRIC_CONSUMPTION_LIVING_WORKER_FACTOR";
    const ELE_CONSUM_WORKER_FACTOR_NIGHT: &'static str = "ELETRIC_CONSUMPTION_LIGHTING_WORKER_FACTOR";
    const ELE_CONSUM_SERVE_FACTOR_BASE:   &'static str = "ELETRIC_CONSUMPTION_LIVING_WORKER_FACTOR_ABLE_SERVE";
    const ELE_CONSUM_SERVE_FACTOR_NIGHT:  &'static str = "ELETRIC_CONSUMPTION_LIGHTING_WORKER_FACTOR_ABLE_SERVE";

    const ELE_CONSUM_CARGO_LOAD_FACTOR:   &'static str = "ELETRIC_CONSUMPTION_LOADING_FIXED";
    const ELE_CONSUM_CARGO_UNLOAD_FACTOR: &'static str = "ELETRIC_CONSUMPTION_UNLOADING_FIXED";
    const NO_ELE_WORK_FACTOR_BASE:        &'static str = "ELETRIC_WITHOUT_WORKING_FACTOR";
    const NO_ELE_WORK_FACTOR_NIGHT:       &'static str = "ELETRIC_WITHOUT_LIGHTING_FACTOR";
    const NO_HEAT_WORK_FACTOR:            &'static str = "HEATING_WITHOUT_WORKING_FACTOR";


    const ENGINE_SPEED:                   &'static str = "ENGINE_SPEED";
    const CABLEWAY_HEAVY:                 &'static str = "CABLEWAY_HEAVY";
    const CABLEWAY_LIGHT:                 &'static str = "CABLEWAY_LIGHT";
    const RESOURCE_SOURCE:                &'static str = "RESOURCE_SOURCE_";

    const STORAGE:                        &'static str = "STORAGE";
    const STORAGE_SPECIAL:                &'static str = "STORAGE_SPECIAL";
    const STORAGE_FUEL:                   &'static str = "STORAGE_FUEL";
    const STORAGE_EXPORT:                 &'static str = "STORAGE_EXPORT";
    const STORAGE_IMPORT:                 &'static str = "STORAGE_IMPORT";
    const STORAGE_IMPORT_CARPLANT:        &'static str = "STORAGE_IMPORT_CARPLANT";
    const STORAGE_EXPORT_SPECIAL:         &'static str = "STORAGE_EXPORT_SPECIAL";
    const STORAGE_IMPORT_SPECIAL:         &'static str = "STORAGE_IMPORT_SPECIAL";
    const STORAGE_DEMAND_BASIC:           &'static str = "STORAGE_DEMAND_BASIC";
    const STORAGE_DEMAND_MEDIUMADVANCED:  &'static str = "STORAGE_DEMAND_MEDIUMADVANCED";
    const STORAGE_DEMAND_ADVANCED:        &'static str = "STORAGE_DEMAND_ADVANCED";
    const STORAGE_DEMAND_HOTEL:           &'static str = "STORAGE_DEMAND_HOTEL";
    const STORAGE_PACK_FROM:              &'static str = "STORAGE_PACKCONTAINERS_FROM_STORAGE";
    const STORAGE_UNPACK_TO:              &'static str = "STORAGE_UNPACKCONTAINERS_TO_STORAGE";
    const STORAGE_LIVING_AUTO:            &'static str = "STORAGE_LIVING_AUTO";

    const VEHICLE_LOADING_FACTOR:         &'static str = "VEHICLE_LOADING_FACTOR";
    const VEHICLE_UNLOADING_FACTOR:       &'static str = "VEHICLE_UNLOADING_FACTOR";
    const VEHICLE_CANNOT_SELECT:          &'static str = "VEHICLE_CANNOTSELECT_INSIDE";
    const LONG_TRAINS:                    &'static str = "LONG_TRAINS";

    const ROAD_VEHICLE_NOT_FLIP:          &'static str = "ROADVEHICLE_NOTFLIP";
    const ROAD_VEHICLE_ELECTRIC:          &'static str = "ROADVEHICLE_ELETRIC";

    const WORKING_VEHICLES_NEEDED:        &'static str = "WORKING_VEHICLES_NEEDED";
    const VEHICLE_STATION:                &'static str = "VEHICLE_STATION";
    const VEHICLE_STATION_NOT_BLOCK:      &'static str = "STATION_NOT_BLOCK";
    const VEHICLE_STATION_DETOUR_POINT:   &'static str = "STATION_NOT_BLOCK_DETOUR_POINT";
    const VEHICLE_STATION_DETOUR_PID:     &'static str = "STATION_NOT_BLOCK_DETOUR_POINT_PID";

    const VEHICLE_PARKING:                &'static str = "VEHICLE_PARKING";
    const VEHICLE_PARKING_DETOUR_POINT:   &'static str = "VEHICLE_PARKING_ADVANCED_POINT";
    const VEHICLE_PARKING_DETOUR_PID:     &'static str = "VEHICLE_PARKING_ADVANCED_POINT_PID";
    const VEHICLE_PARKING_PERSONAL:       &'static str = "VEHICLE_PARKING_PERSONAL";

    const AIRPLANE_STATION:               &'static str = "AIRPLANE_STATION_";
    const HELIPORT_STATION:               &'static str = "HELIPORT_STATION";
    const SHIP_STATION:                   &'static str = "SHIP_STATION";
    const HELIPORT_AREA:                  &'static str = "HELIPORT_AREA";
    const HARBOR_OVER_TERRAIN_FROM:       &'static str = "HARBOR_OVER_TERRAIN_FROM";
    const HARBOR_OVER_WATER_FROM:         &'static str = "HARBOR_OVER_WATER_FROM";
    const HARBOR_EXTEND_WHEN_BULDING:     &'static str = "HARBOR_EXTEND_AREA_WHEN_BULDING";

    const CONNECTION:                     &'static str = "CONNECTION_";
    const OFFSET_CONNECTION_XYZW:         &'static str = "OFFSET_CONNECTION_XYZW";
    const CONNECTION_RAIL_DEADEND:        &'static str = "RAIL_DEADEND";

    const CONNECTIONS_SPACE:               &'static str = "CONNECTIONS_SPACE";
    const CONNECTIONS_ROAD_DEAD_SQUARE:    &'static str = "CONNECTIONS_ROAD_DEAD_SQUARE";
    const CONNECTIONS_AIRPORT_DEAD_SQUARE: &'static str = "CONNECTIONS_AIRPORT_DEAD_SQUARE";
    const CONNECTIONS_WATER_DEAD_SQUARE:   &'static str = "CONNECTIONS_WATER_DEAD_SQUARE";

    const ATTRACTION_TYPE:                 &'static str = "ATTRACTIVE_TYPE_";
    const ATTRACTION_REMEMBER_USAGE:       &'static str = "ATTRACTIVE_USE_FORGOT_EVEN_MATCH";
    const ATTRACTIVE_SCORE_BASE:           &'static str = "ATTRACTIVE_SCORE";
    const ATTRACTIVE_SCORE_ALCOHOL:        &'static str = "ATTRACTIVE_SCORE_ALCOHOL";
    const ATTRACTIVE_SCORE_CULTURE:        &'static str = "ATTRACTIVE_SCORE_CULTURE";
    const ATTRACTIVE_SCORE_RELIGION:       &'static str = "ATTRACTIVE_SCORE_RELIGION";
    const ATTRACTIVE_SCORE_SPORT:          &'static str = "ATTRACTIVE_SCORE_SPORT";
    const ATTRACTIVE_FACTOR_NATURE:        &'static str = "ATTRACTIVE_FACTOR_NATURE";
    const ATTRACTIVE_FACTOR_NATURE_ADD:    &'static str = "ATTRACTIVE_FACTOR_NATURE_ADD";
    const ATTRACTIVE_FACTOR_POLLUTION:     &'static str = "ATTRACTIVE_FACTOR_POLLUTION";
    const ATTRACTIVE_FACTOR_POLLUTION_ADD: &'static str = "ATTRACTIVE_FACTOR_POLLUTION_ADD";
    const ATTRACTIVE_FACTOR_SIGHT:         &'static str = "ATTRACTIVE_FACTOR_SIGHT";
    const ATTRACTIVE_FACTOR_SIGHT_ADD:     &'static str = "ATTRACTIVE_FACTOR_SIGHT_ADD";
    const ATTRACTIVE_FACTOR_WATER:         &'static str = "ATTRACTIVE_FACTOR_WATER";
    const ATTRACTIVE_FACTOR_WATER_ADD:     &'static str = "ATTRACTIVE_FACTOR_WATER_ADD";

    const POLLUTION_HIGH:                 &'static str = "POLLUTION_HIGH";
    const POLLUTION_MEDIUM:               &'static str = "POLLUTION_MEDIUM";
    const POLLUTION_SMALL:                &'static str = "POLLUTION_SMALL";

    const PARTICLE:                       &'static str = "PARTICLE";
    const PARTICLE_REACTOR:               &'static str = "PARTICLE_REACTOR";
    const TEXT_CAPTION:                   &'static str = "TEXT_CAPTION";
    const WORKER_RENDERING_AREA:          &'static str = "WORKER_RENDERING_AREA";
    const RESOURCE_VISUALIZATION:         &'static str = "RESOURCE_VISUALIZATION";
    const RESOURCE_INCREASE_POINT:        &'static str = "RESOURCE_INCREASE_POINT";
    const RESOURCE_INCREASE_CONV_POINT:   &'static str = "RESOURCE_INCREASE_CONVEYOR_POINT";
    const RESOURCE_FILLING_POINT:         &'static str = "RESOURCE_FILLING_POINT";
    const RESOURCE_FILLING_CONV_POINT:    &'static str = "RESOURCE_FILLING_CONVEYOR_POINT";
    const WORKING_SFX:                    &'static str = "WORKING_SFX";
    const ANIMATION_FPS:                  &'static str = "ANIMATION_SPEED_FPS";
    const ANIMATION_MESH:                 &'static str = "ANIMATION_MESH";
    const UNDERGROUND_MESH:               &'static str = "UNDERGROUND_MESH";


    const COST_WORK:                      &'static str = "COST_WORK";
    const COST_WORK_BUILDING_NODE:        &'static str = "COST_WORK_BUILDING_NODE";
    const COST_WORK_BUILDING_ALL:         &'static str = "COST_WORK_BUILDING_ALL";
    const COST_WORK_BUILDING_KEYWORD:     &'static str = "COST_WORK_BUILDING_KEYWORD";

    const COST_RESOURCE:                  &'static str = "COST_RESOURCE";
    const COST_RESOURCE_AUTO:             &'static str = "COST_RESOURCE_AUTO";
    const COST_WORK_VEHICLE_STATION:      &'static str = "COST_WORK_VEHICLE_STATION";
    const COST_WORK_VEHICLE_STATION_NODE: &'static str = "COST_WORK_VEHICLE_STATION_ACCORDING_NODE";
}


impl<'t> super::IniToken for Token<'t> {
    fn serialize<W: std::io::Write>(&self, wr: W) -> Result<(), std::io::Error> {
        self.serialize_token(wr)
    }
}


#[derive(Clone)]
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
    GarbageOffice,
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
    PedestrianBridge,
    PoliceStation,
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
    const TYPE_AIRPLANE_GATE:            &'static str = "AIRPLANE_GATE";
    const TYPE_AIRPLANE_PARKING:         &'static str = "AIRPLANE_PARKING";
    const TYPE_AIRPLANE_TOWER:           &'static str = "AIRPLANE_TOWER";
    const TYPE_ATTRACTION:               &'static str = "ATTRACTION";
    const TYPE_BROADCAST:                &'static str = "BROADCAST";
    const TYPE_CAR_DEALER:               &'static str = "CAR_DEALER";
    const TYPE_CARGO_STATION:            &'static str = "CARGO_STATION";
    const TYPE_CHURCH:                   &'static str = "CHURCH";
    const TYPE_CITYHALL:                 &'static str = "CITYHALL";
    const TYPE_CONSTRUCTION_OFFICE:      &'static str = "CONSTRUCTION_OFFICE";
    const TYPE_CONSTRUCTION_OFFICE_RAIL: &'static str = "CONSTRUCTION_OFFICE_RAIL";
    const TYPE_CONTAINER_FACILITY:       &'static str = "CONTAINER_FACILITY";
    const TYPE_COOLING_TOWER:            &'static str = "COOLING_TOWER";
    const TYPE_CUSTOMHOUSE:              &'static str = "CUSTOMHOUSE";
    const TYPE_DISTRIBUTION_OFFICE:      &'static str = "DISTRIBUTION_OFFICE";
    const TYPE_ELETRIC_EXPORT:           &'static str = "ELETRIC_EXPORT";
    const TYPE_ELETRIC_IMPORT:           &'static str = "ELETRIC_IMPORT";
    const TYPE_ENGINE:                   &'static str = "ENGINE";
    const TYPE_FACTORY:                  &'static str = "FACTORY";
    const TYPE_FARM:                     &'static str = "FARM";
    const TYPE_FIELD:                    &'static str = "FIELD";
    const TYPE_FIRESTATION:              &'static str = "FIRESTATION";
    const TYPE_FORKLIFT_GARAGE:          &'static str = "FORKLIFT_GARAGE"; 
    const TYPE_GARBAGE_OFFICE:           &'static str = "GARBAGE_OFFICE"; 
    const TYPE_GAS_STATION:              &'static str = "GAS_STATION";
    const TYPE_HEATING_ENDSTATION:       &'static str = "HEATING_ENDSTATION";
    const TYPE_HEATING_PLANT:            &'static str = "HEATING_PLANT";
    const TYPE_HEATING_SWITCH:           &'static str = "HEATING_SWITCH";
    const TYPE_HOSPITAL:                 &'static str = "HOSPITAL";
    const TYPE_HOTEL:                    &'static str = "HOTEL";
    const TYPE_KINDERGARTEN:             &'static str = "KINDERGARTEN";
    const TYPE_KINO:                     &'static str = "KINO";
    const TYPE_LIVING:                   &'static str = "LIVING";
    const TYPE_MINE_BAUXITE:             &'static str = "MINE_BAUXITE";
    const TYPE_MINE_COAL:                &'static str = "MINE_COAL";
    const TYPE_MINE_GRAVEL:              &'static str = "MINE_GRAVEL";
    const TYPE_MINE_IRON:                &'static str = "MINE_IRON";
    const TYPE_MINE_OIL:                 &'static str = "MINE_OIL";
    const TYPE_MINE_URANIUM:             &'static str = "MINE_URANIUM";
    const TYPE_MINE_WOOD:                &'static str = "MINE_WOOD";
    const TYPE_MONUMENT:                 &'static str = "MONUMENT";
    const TYPE_PARKING:                  &'static str = "PARKING";
    const TYPE_PASSANGER_STATION:        &'static str = "PASSANGER_STATION";
    const TYPE_PEDESTRIAN_BRIDGE:        &'static str = "PEDESTRIAN_BRIDGE";
    const TYPE_POLICE_STATION:           &'static str = "POLICE_STATION";
    const TYPE_POLLUTION_METER:          &'static str = "POLLUTION_METER";
    const TYPE_POWERPLANT:               &'static str = "POWERPLANT";
    const TYPE_PRODUCTION_LINE:          &'static str = "PRODUCTION_LINE";
    const TYPE_PUB:                      &'static str = "PUB";
    const TYPE_RAIL_TRAFO:               &'static str = "RAIL_TRAFO";
    const TYPE_RAILDEPO:                 &'static str = "RAILDEPO";
    const TYPE_ROADDEPO:                 &'static str = "ROADDEPO";
    const TYPE_SCHOOL:                   &'static str = "SCHOOL";
    const TYPE_SHIP_DOCK:                &'static str = "SHIP_DOCK";
    const TYPE_SHOP:                     &'static str = "SHOP";
    const TYPE_SPORT:                    &'static str = "SPORT";
    const TYPE_STORAGE:                  &'static str = "STORAGE";
    const TYPE_SUBSTATION:               &'static str = "SUBSTATION";
    const TYPE_TRANSFORMATOR:            &'static str = "TRANSFORMATOR";
    const TYPE_UNIVERSITY:               &'static str = "UNIVERSITY";
}


#[derive(Clone)]
pub enum BuildingSubtype {
    Aircustom,
    Airplane,
    Cableway,
    Hostel,
    Medical,
    Radio,
    Rail,
    Restaurant,
    Road,
    Ship,
    Soviet,
    SpaceForVehicles,
    Technical,
    Television,
    Trolleybus,
}


impl BuildingSubtype {
    const SUBTYPE_AIRCUSTOM:          &'static str = "AIRCUSTOM";
    const SUBTYPE_AIRPLANE:           &'static str = "AIRPLANE";
    const SUBTYPE_CABLEWAY:           &'static str = "CABLEWAY";
    const SUBTYPE_HOSTEL:             &'static str = "HOSTEL";
    const SUBTYPE_MEDICAL:            &'static str = "MEDICAL";
    const SUBTYPE_RADIO:              &'static str = "RADIO";
    const SUBTYPE_RAIL:               &'static str = "RAIL";
//  const SUBTYPE_RAL:                &'static str = "RAL";  probably a typo from early release?
    const SUBTYPE_RESTAURANT:         &'static str = "RESTAURANT";
    const SUBTYPE_ROAD:               &'static str = "ROAD";
    const SUBTYPE_SHIP:               &'static str = "SHIP";
    const SUBTYPE_SOVIET:             &'static str = "SOVIET";
    const SUBTYPE_SPACE_FOR_VEHICLES: &'static str = "SPACE_FOR_VEHICLES";
    const SUBTYPE_TECHNICAL:          &'static str = "TECHNICAL";
    const SUBTYPE_TELEVISION:         &'static str = "TELEVISION";
    const SUBTYPE_TROLLEYBUS:         &'static str = "TROLLEYBUS";
}


#[derive(Clone)]
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
    Vehicles,
    Nuclear1,
    Nuclear2,
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
    const VEHICLES:  &'static str = "RESOURCE_TRANSPORT_VEHICLES";
    //const ELETRIC:   &'static str = "RESOURCE_TRANSPORT_ELETRIC";
    //const HEATING:   &'static str = "RESOURCE_TRANSPORT_HEATING";
    const NUCLEAR1:  &'static str = "RESOURCE_TRANSPORT_NUCLEAR1";
    const NUCLEAR2:  &'static str = "RESOURCE_TRANSPORT_NUCLEAR2";
}


#[derive(Clone, Copy)]
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
    Fountain1,
    Fountain2,
    Fountain3,
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
    const FOUNTAIN_1          : &'static str = "fountain1";
    const FOUNTAIN_2          : &'static str = "fountain2";
    const FOUNTAIN_3          : &'static str = "fountain3";
}


#[derive(Clone)]
pub enum ConstructionPhase {
    AsphaltLaying,
    AsphaltRolling,
    BoardsLaying,
    BricksLaying,
    BridgeBuilding,
    GravelLaying,
    Groundworks,    
    InteriorWorks,
    PanelsLaying,
    RailwayLaying,
    RooftopBuilding,
    SkeletonCasting,
    SteelLaying,
    Tunneling,
    WireLaying,
}


impl ConstructionPhase {
    const ASPHALT_LAYING:   &'static str = "SOVIET_CONSTRUCTION_ASPHALT_LAYING";
    const ASPHALT_ROLLING:  &'static str = "SOVIET_CONSTRUCTION_ASPHALT_ROLLING";
    const BOARDS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_BOARDS_LAYING";
    const BRICKS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_BRICKS_LAYING";
    const BRIDGE_BUILDING:  &'static str = "SOVIET_CONSTRUCTION_BRIDGE_BUILDING";
    const GRAVEL_LAYING:    &'static str = "SOVIET_CONSTRUCTION_GRAVEL_LAYING";
    const GROUNDWORKS:      &'static str = "SOVIET_CONSTRUCTION_GROUNDWORKS";
    const INTERIOR_WORKS:   &'static str = "SOVIET_CONSTRUCTION_INTERIOR_WORKS";
    const PANELS_LAYING:    &'static str = "SOVIET_CONSTRUCTION_PANELS_LAYING";
    const RAILWAY_LAYING:   &'static str = "SOVIET_CONSTRUCTION_RAILWAY_LAYING";
    const ROOFTOP_BUILDING: &'static str = "SOVIET_CONSTRUCTION_ROOFTOP_BUILDING";
    const SKELETON_CASTING: &'static str = "SOVIET_CONSTRUCTION_SKELETON_CASTING";
    const STEEL_LAYING:     &'static str = "SOVIET_CONSTRUCTION_STEEL_LAYING";
    const TUNNELING:        &'static str = "SOVIET_CONSTRUCTION_TUNNELING";
    const WIRE_LAYING:      &'static str = "SOVIET_CONSTRUCTION_WIRE_LAYING";
}


#[derive(Clone)]
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
    TechElectroSteel,
    RoofWoodBrick,
    RoofSteel,
    RoofWoodSteel
}


impl ConstructionAutoCost {
    const GROUND:             &'static str = "ground";
    const GROUND_ASPHALT:     &'static str = "ground_asphalt";
    const WALL_CONCRETE:      &'static str = "wall_concrete";
    const WALL_PANELS:        &'static str = "wall_panels";
    const WALL_BRICK:         &'static str = "wall_brick";
    const WALL_STEEL:         &'static str = "wall_steel";
    const WALL_WOOD:          &'static str = "wall_wood";
    const TECH_STEEL:         &'static str = "tech_steel";
    const ELECTRO_STEEL:      &'static str = "electro_steel";
    const TECH_ELECTRO_STEEL: &'static str = "techelectro_steel";
    const ROOF_WOOD_BRICK:    &'static str = "roof_woodbrick";
    const ROOF_STEEL:         &'static str = "roof_steel";
    const ROOF_WOOD_STEEL:    &'static str = "roof_woodsteel";
}


#[derive(Clone)]
pub enum ResourceType {
    Alcohol,
    Alumina,
    Aluminium,
    Asphalt,
    Bauxite,
    Bitumen,
    Boards,
    Bricks,
    Cement,
    Chemicals,
    Clothes,
    Coal,
    Concrete,
    Crops,
    ElectroComponents,
    Electricity,
    Electronics,
    Fabric,
    Food,
    Fuel,
    Gravel,
    Heat,
    Iron,
    Livestock,
    MechComponents,
    Meat,
    NuclearFuel,
    NuclearWaste,
    Oil,
    Plastic,
    PrefabPanels,
    RawBauxite,
    RawCoal,
    RawGravel,
    RawIron,
    Steel,
    UF6,
    Uranium,
    Vehicles,
    Wood,
    Workers,
    Yellowcake,
}


impl ResourceType {
    const ALCOHOL:       &'static str = "alcohol";
    const ALUMINA:       &'static str = "alumina";
    const ALUMINIUM:     &'static str = "aluminium";
    const ASPHALT:       &'static str = "asphalt";
    const BAUXITE:       &'static str = "bauxite";
    const BITUMEN:       &'static str = "bitumen";
    const BOARDS:        &'static str = "boards";
    const BRICKS:        &'static str = "bricks";
    const CEMENT:        &'static str = "cement";
    const CHEMICALS:     &'static str = "chemicals";
    const CLOTHES:       &'static str = "clothes";
    const COAL:          &'static str = "coal";
    const CONCRETE:      &'static str = "concrete";
    const CROPS:         &'static str = "plants";
    const ELECTRO_COMP:  &'static str = "ecomponents";
    const ELECTRICITY:   &'static str = "eletric";
    const ELECTRONICS:   &'static str = "eletronics";
    const FABRIC:        &'static str = "fabric";
    const FOOD:          &'static str = "food";
    const FUEL:          &'static str = "fuel";
    const GRAVEL:        &'static str = "gravel";
    const HEAT:          &'static str = "heat";
    const IRON:          &'static str = "iron";
    const LIVESTOCK:     &'static str = "livestock";
    const MECH_COMP:     &'static str = "mcomponents";
    const MEAT:          &'static str = "meat";
    const NUCLEAR_FUEL:  &'static str = "nuclearfuel";
    const NUCLEAR_WASTE: &'static str = "nuclearfuelburned";
    const OIL:           &'static str = "oil";
    const PLASTIC:       &'static str = "plastics";
    const PREFABS:       &'static str = "prefabpanels";
    const RAW_BAUXITE:   &'static str = "rawbauxite";
    const RAW_COAL:      &'static str = "rawcoal";
    const RAW_GRAVEL:    &'static str = "rawgravel";
    const RAW_IRON:      &'static str = "rawiron";
    const STEEL:         &'static str = "steel";
    const UF_6:          &'static str = "uf6";
    const URANIUM:       &'static str = "uranium";
    const VEHICLES:      &'static str = "vehicles";
    const WOOD:          &'static str = "wood";
    const WORKERS:       &'static str = "workers";
    const YELLOWCAKE:    &'static str = "yellowcake";
}


#[derive(Clone, Copy)]
pub enum Connection2PType {
    AirRoad,
    Pedestrian,
    PedestrianNotPick,
    Road,
    RoadAllowpass,
    RoadBorder,
    RoadIn,
    RoadOut,
    Rail,
    RailAllowpass,
    RailBorder,
    RailHeight,
    HeatingBig,
    HeatingSmall,
    SteamIn,
    SteamOut,
    PipeIn,
    PipeOut,
    BulkIn,
    BulkOut,
    Cableway,
    Factory,
    ConveyorIn,
    ConveyorOut,
    ElectricHighIn,
    ElectricHighOut,
    ElectricLowIn,
    ElectricLowOut,
    Fence,
}

impl Connection2PType {
    const CONN_AIRROAD:        &'static str = "AIRROAD";
    const CONN_PED:            &'static str = "PEDESTRIAN";
    const CONN_PED_NOTPICK:    &'static str = "PEDESTRIAN_NOTPICK";
    const CONN_ROAD:           &'static str = "ROAD";
    const CONN_ROAD_ALLOWPASS: &'static str = "ROAD_ALLOWPASS";
    const CONN_ROAD_BORDER:    &'static str = "ROAD_BORDER";
    const CONN_ROAD_IN:        &'static str = "ROAD_INPUT";
    const CONN_ROAD_OUT:       &'static str = "ROAD_OUTPUT";
    const CONN_RAIL:           &'static str = "RAIL";
    const CONN_RAIL_ALLOWPASS: &'static str = "RAIL_ALLOWPASS";
    const CONN_RAIL_BORDER:    &'static str = "RAIL_BORDER";
    const CONN_RAIL_HEIGHT:    &'static str = "RAIL_HEIGHT";
    const CONN_HEATING_BIG:    &'static str = "HEATING_BIG";
    const CONN_HEATING_SMALL:  &'static str = "HEATING_SMALL";
    const CONN_STEAM_IN:       &'static str = "STEAM_INPUT";
    const CONN_STEAM_OUT:      &'static str = "STEAM_OUTPUT";
    const CONN_PIPE_IN:        &'static str = "PIPE_INPUT";
    const CONN_PIPE_OUT:       &'static str = "PIPE_OUTPUT";
    const CONN_BULK_IN:        &'static str = "BULK_INPUT";
    const CONN_BULK_OUT:       &'static str = "BULK_OUTPUT";
    const CONN_CABLEWAY:       &'static str = "CABLEWAY";
    const CONN_FACTORY:        &'static str = "CONNECTION";
    const CONN_CONVEYOR_IN:    &'static str = "CONVEYOR_INPUT";
    const CONN_CONVEYOR_OUT:   &'static str = "CONVEYOR_OUTPUT";
    const CONN_ELECTRIC_H_IN:  &'static str = "ELETRIC_HIGH_INPUT";
    const CONN_ELECTRIC_H_OUT: &'static str = "ELETRIC_HIGH_OUTPUT";
    const CONN_ELECTRIC_L_IN:  &'static str = "ELETRIC_LOW_INPUT";
    const CONN_ELECTRIC_L_OUT: &'static str = "ELETRIC_LOW_OUTPUT";
    const CONN_FENCE:          &'static str = "FENCE";
}


#[derive(Clone, Copy)] 
pub enum Connection1PType {
    RoadDead,
    PedestrianDead,
    WaterDead,
    AirportDead,
    AdvancedPoint,
}


impl Connection1PType {
    const ROAD_DEAD:       &'static str = "ROAD_DEAD";
    const PEDESTRIAN_DEAD: &'static str = "PEDESTRIAN_DEAD";
    const WATER_DEAD:      &'static str = "WATER_DEAD";
    const AIRPORT_DEAD:    &'static str = "AIRPORT_DEAD";
    const ADVANCED_POINT:  &'static str = "ADVANCED_POINT";
}


#[derive(Clone, Copy)]
pub enum AirplaneStationType {
    M30,
    M40,
    M50,
    M75,
}

impl AirplaneStationType {
    const AIRPLANE_STATION_30M: &'static str = "30M";
    const AIRPLANE_STATION_40M: &'static str = "40M";
    const AIRPLANE_STATION_50M: &'static str = "50M";
    const AIRPLANE_STATION_75M: &'static str = "75M";
}

#[derive(Clone, Copy)]
pub enum AttractionType {
    Carousel,
    Gallery,
    Museum,
    Sight,
    Swim,
    Zoo,
}

impl AttractionType {
    const ATTRACTION_TYPE_CARUSEL: &'static str = "CARUSEL";
    const ATTRACTION_TYPE_GALLERY: &'static str = "GALLERY";
    const ATTRACTION_TYPE_MUSEUM:  &'static str = "MUSEUM";
    const ATTRACTION_TYPE_SIGHT:   &'static str = "SIGHT";
    const ATTRACTION_TYPE_SWIM:    &'static str = "SWIM";
    const ATTRACTION_TYPE_ZOO:     &'static str = "ZOO";
}


pub enum ResourceSourceType {
    Asphalt,
    Concrete,
    Covered,
    CoveredElectro,
    Gravel,
    Open,
    OpenBoards,
    OpenBricks,
    OpenPanels,
    Workers,
}


impl ResourceSourceType {
    const RES_SOURCE_ASPHALT:         &'static str = "ASPHALT";
    const RES_SOURCE_CONCRETE:        &'static str = "CONCRETE";
    const RES_SOURCE_COVERED:         &'static str = "COVERED";
    const RES_SOURCE_COVERED_ELECTRO: &'static str = "COVERED_ELECTRO";
    const RES_SOURCE_GRAVEL:          &'static str = "GRAVEL";
    const RES_SOURCE_OPEN:            &'static str = "OPEN";
    const RES_SOURCE_OPEN_BOARDS:     &'static str = "OPEN_BOARDS";
    const RES_SOURCE_OPEN_BRICKS:     &'static str = "OPEN_BRICKS";
    const RES_SOURCE_OPEN_PANELS:     &'static str = "OPEN_PANELS";
    const RES_SOURCE_WORKERS:         &'static str = "WORKERS";
}


#[derive(Clone)]
pub struct ResourceVisualization {
    pub storage_id: u32,
    pub position: Point3f,
    pub rotation: f32,
    pub scale: Point3f,
    pub numstep_x: (f32, u32),
    pub numstep_z: (f32, u32),
}
