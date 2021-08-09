use std::fmt::{Formatter, Error, Display};
use std::io::{Write};

use super::{BuildingType,
            BuildingSubtype,
            ResourceVisualization,
            Token,
            Point3f,
            Tagged2Points,
           };


type IOResult = Result<(), std::io::Error>;


impl Token<'_> {
    pub fn serialize_token<W: Write>(&self, mut wr: W) -> IOResult {
        #[inline]
        fn write_pfx_pt<W: Write>(mut wr: W, pfx: &str, a: &Point3f) -> IOResult {
            write!(wr, "{}\r\n{} {} {}", pfx, a.x, a.y, a.z)
        }

        #[inline]
        fn write_pfx_2pts<W: Write>(mut wr: W, tag: &str, a: &Point3f, b: &Point3f) -> IOResult {
            write!(wr, "{}\r\n{} {} {}\r\n{} {} {}", tag, a.x, a.y, a.z, b.x, b.y, b.z)
        }

        #[inline]
        fn write_pfx_tag2pts<W: Write, T: Display>(mut wr: W, prefix: &str, tpp: &Tagged2Points<T>) -> IOResult {
            let Tagged2Points { tag, p1, p2 } = tpp;
            write!(wr, "{}{}\r\n{} {} {}\r\n{} {} {}", prefix, tag, p1.x, p1.y, p1.z, p2.x, p2.y, p2.z)
        }

        match self {
            Self::VehicleStationNotBlockDetourPoint(p)         => write_pfx_pt(wr, Self::VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT, p),
            Self::VehicleStationNotBlockDetourPointPid((i, p)) => write!(wr, "{} {} {} {} {}", Self::VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT_PID, i, p.x, p.y, p.z),
            Self::VehicleStation((a, b))           => write_pfx_2pts(wr, Self::VEHICLE_STATION, a, b),
            Self::VehicleParking((a, b))           => write_pfx_2pts(wr, Self::VEHICLE_PARKING, a, b),
            Self::AirplaneStation(tpp)             => write_pfx_tag2pts(wr, Self::AIRPLANE_STATION, tpp),

            Self::Connection2Points(tpp)           => write_pfx_tag2pts(wr, Self::CONNECTION, tpp),

            Self::ConnectionRoadDead(x)            => write!(wr, "{}{}\r\n{}", Self::CONNECTION, Self::CONNECTION_ROAD_DEAD, x),
            Self::ConnectionAirportDead(x)         => write!(wr, "{}{}\r\n{}", Self::CONNECTION, Self::CONNECTION_AIRPORT_DEAD, x),
            Self::ConnectionAdvancedPoint(x)       => write!(wr, "{}{}\r\n{}", Self::CONNECTION, Self::CONNECTION_ADVANCED_POINT, x),

            Self::ConnectionsSpace(r)              => write!(wr, "{}\r\n{} {}\r\n{} {}", Self::CONNECTIONS_SPACE,               r.x1, r.z1, r.x2, r.z2),
            Self::ConnectionsRoadDeadSquare(r)     => write!(wr, "{}\r\n{} {}\r\n{} {}", Self::CONNECTIONS_ROAD_DEAD_SQUARE,    r.x1, r.z1, r.x2, r.z2),
            Self::ConnectionsAirportDeadSquare(r)  => write!(wr, "{}\r\n{} {}\r\n{} {}", Self::CONNECTIONS_AIRPORT_DEAD_SQUARE, r.x1, r.z1, r.x2, r.z2),

            Self::Particle((t, p, a, s))           => write!(wr, "{} {} {} {} {} {} {}", Self::PARTICLE, t, p.x, p.y, p.z, a, s),
            Self::TextCaption((a, b))              => write_pfx_2pts(wr, Self::TEXT_CAPTION, a, b),
            Self::WorkerRenderingArea((a, b))      => write_pfx_2pts(wr, Self::WORKER_RENDERING_AREA, a, b),
            Self::ResourceVisualization(ResourceVisualization { storage_id, position: p, rotation, scale: s, numstep_x: (x1, x2), numstep_z: (z1, z2) }) => 
                write!(wr, "{} {}\nposition {} {} {}\nrotation {}\nscale {} {} {}\nnumstep_x {} {}\nnumstep_t {} {}", 
                       Self::RESOURCE_VISUALIZATION, storage_id, p.x, p.y, p.z, rotation, s.x, s.y, s.z, x1, x2, z1, z2),

            Self::CostWorkVehicleStation((a, b))   => write_pfx_2pts(wr, Self::COST_WORK_VEHICLE_STATION, a, b),

            t => write!(wr, "{}", t)
        }

    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::NameStr(p)                       => write!(f, "{} {}", Self::NAME_STR, p),
            Self::Name(p)                          => write!(f, "{} {}", Self::NAME, p),
            Self::BuildingType(p)                  => write!(f, "{}{}",  Self::BUILDING_TYPE, p),
            Self::BuildingSubtype(p)               => write!(f, "{}{}",  Self::BUILDING_SUBTYPE, p),

            Self::HeatEnable                       => write!(f, "{}",    Self::HEATING_ENABLE),
            Self::HeatDisable                      => write!(f, "{}",    Self::HEATING_DISABLE),
            Self::CivilBuilding                    => write!(f, "{}",    Self::CIVIL_BUILDING),
            Self::QualityOfLiving(x)               => write!(f, "{} {}", Self::QUALITY_OF_LIVING, x),

            Self::WorkersNeeded(x)                 => write!(f, "{} {}",    Self::WORKERS_NEEDED, x),
            Self::ProfessorsNeeded(x)              => write!(f, "{} {}",    Self::PROFESSORS_NEEDED, x),
            Self::CitizenAbleServe(x)              => write!(f, "{} {}",    Self::CITIZEN_ABLE_SERVE, x),
            Self::Consumption((t, x))              => write!(f, "{} {} {}", Self::CONSUMPTION, t, x),
            Self::ConsumptionPerSecond((t, x))     => write!(f, "{} {} {}", Self::CONSUMPTION_PER_SECOND, t, x),
            Self::Production((t, x))               => write!(f, "{} {} {}", Self::PRODUCTION, t, x),

            Self::EleConsumLightingWorkerFactor(x) => write!(f, "{} {}", Self::ELE_CONSUM_LIGHTING_WORKER_FACTOR, x),

            Self::Storage((t, x))                  => write!(f, "{} {} {}", Self::STORAGE, t, x),
            Self::StorageFuel((t, x))              => write!(f, "{} {} {}", Self::STORAGE_FUEL, t, x),
            Self::StorageExport((t, x))            => write!(f, "{} {} {}", Self::STORAGE_EXPORT, t, x),
            Self::StorageImport((t, x))            => write!(f, "{} {} {}", Self::STORAGE_IMPORT, t, x),
            Self::StorageExportSpecial((t, x, r))  => write!(f, "{} {} {} {}", Self::STORAGE_EXPORT, t, x, r),
            Self::StorageImportSpecial((t, x, r))  => write!(f, "{} {} {} {}", Self::STORAGE_IMPORT, t, x, r),

            Self::RoadNotFlip                      => write!(f, "{}", Self::ROAD_VEHICLE_NOT_FLIP),
            Self::RoadElectric                     => write!(f, "{}", Self::ROAD_VEHICLE_ELECTRIC),
            Self::VehicleStationNotBlock           => write!(f, "{}", Self::VEHICLE_STATION_NOT_BLOCK),
            Self::VehicleStationNotBlockDetourPoint(p)         => write!(f, "{} {}",    Self::VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT, p),
            Self::VehicleStationNotBlockDetourPointPid((i, p)) => write!(f, "{} {} {}", Self::VEHICLE_STATION_NOT_BLOCK_DETOUR_POINT_PID, i, p),
            Self::VehicleStation((a, b))           => write!(f, "{} {} {}", Self::VEHICLE_STATION, a, b),
            Self::WorkingVehiclesNeeded(x)         => write!(f, "{} {}",    Self::WORKING_VEHICLES_NEEDED, x),
            Self::VehicleParking((a, b))           => write!(f, "{} {} {}", Self::VEHICLE_PARKING, a, b),

            Self::AirplaneStation(tpp)             => write!(f, "{}{}", Self::AIRPLANE_STATION, tpp),
            Self::HeliportArea(x)                  => write!(f, "{} {}", Self::HELIPORT_AREA, x),

            Self::Connection2Points(tpp)           => write!(f, "{}{}", Self::CONNECTION, tpp),

            Self::ConnectionRoadDead(x)            => write!(f, "{}{} {}", Self::CONNECTION, Self::CONNECTION_ROAD_DEAD, x),
            Self::ConnectionAirportDead(x)         => write!(f, "{}{} {}", Self::CONNECTION, Self::CONNECTION_AIRPORT_DEAD, x),
            Self::ConnectionAdvancedPoint(x)       => write!(f, "{}{} {}", Self::CONNECTION, Self::CONNECTION_ADVANCED_POINT, x),

            Self::ConnectionsSpace(r)              => write!(f, "{} {}", Self::CONNECTIONS_SPACE, r),
            Self::ConnectionsRoadDeadSquare(r)     => write!(f, "{} {}", Self::CONNECTIONS_ROAD_DEAD_SQUARE, r),
            Self::ConnectionsAirportDeadSquare(r)  => write!(f, "{} {}", Self::CONNECTIONS_AIRPORT_DEAD_SQUARE, r),

            Self::Particle((t, x, a, s))           => write!(f, "{} {} {} {} {}", Self::PARTICLE, t, x, a, s),
            Self::TextCaption((a, b))              => write!(f, "{} {} {}", Self::TEXT_CAPTION, a, b),
            Self::WorkerRenderingArea((a, b))      => write!(f, "{} {} {}", Self::WORKER_RENDERING_AREA, a, b),
            Self::ResourceVisualization(rv)        => write!(f, "{} {}\nposition: {}\nrotation: {}\nscale: {}\nnumstep_x: {:?}\nnumstep_t: {:?}", 
                                                             Self::RESOURCE_VISUALIZATION, rv.storage_id, rv.position, rv.rotation, rv.scale, rv.numstep_x, rv.numstep_z),

            Self::CostWork((t, x))                 => write!(f, "{} {} {}", Self::COST_WORK, t, x),
            Self::CostWorkBuildingNode(n)          => write!(f, "{} {}", Self::COST_WORK_BUILDING_NODE, n),
            Self::CostWorkBuildingKeyword(n)       => write!(f, "{} {}", Self::COST_WORK_BUILDING_KEYWORD, n),
            Self::CostWorkBuildingAll              => write!(f, "{}", Self::COST_WORK_BUILDING_ALL),

            Self::CostResource((t, x))             => write!(f, "{} {} {}", Self::COST_RESOURCE, t, x),
            Self::CostResourceAuto((t, x))         => write!(f, "{} {} {}", Self::COST_RESOURCE_AUTO, t, x),
            Self::CostWorkVehicleStation((a, b))   => write!(f, "{} {} {}", Self::COST_WORK_VEHICLE_STATION, a, b),
            Self::CostWorkVehicleStationNode(p)    => write!(f, "{} {}", Self::COST_WORK_VEHICLE_STATION_NODE, p),
        }
    }
}


impl Display for BuildingType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::AirplaneGate           => Self::TYPE_AIRPLANE_GATE,
            Self::AirplaneParking        => Self::TYPE_AIRPLANE_PARKING,
            Self::AirplaneTower          => Self::TYPE_AIRPLANE_TOWER,
            Self::Attraction             => Self::TYPE_ATTRACTION,
            Self::Broadcast              => Self::TYPE_BROADCAST,
            Self::CarDealer              => Self::TYPE_CAR_DEALER,
            Self::CargoStation           => Self::TYPE_CARGO_STATION,
            Self::Church                 => Self::TYPE_CHURCH,
            Self::Cityhall               => Self::TYPE_CITYHALL,
            Self::ConstructionOffice     => Self::TYPE_CONSTRUCTION_OFFICE,
            Self::ConstructionOfficeRail => Self::TYPE_CONSTRUCTION_OFFICE_RAIL,
            Self::ContainerFacility      => Self::TYPE_CONTAINER_FACILITY,
            Self::CoolingTower           => Self::TYPE_COOLING_TOWER,
            Self::Customhouse            => Self::TYPE_CUSTOMHOUSE,
            Self::DistributionOffice     => Self::TYPE_DISTRIBUTION_OFFICE,
            Self::ElectricExport         => Self::TYPE_ELETRIC_EXPORT,
            Self::ElectricImport         => Self::TYPE_ELETRIC_IMPORT,
            Self::Engine                 => Self::TYPE_ENGINE,
            Self::Factory                => Self::TYPE_FACTORY,
            Self::Farm                   => Self::TYPE_FARM,
            Self::Field                  => Self::TYPE_FIELD,
            Self::Firestation            => Self::TYPE_FIRESTATION,
            Self::ForkliftGarage         => Self::TYPE_FORKLIFT_GARAGE,
            Self::GarbageOffice          => Self::TYPE_GARBAGE_OFFICE,
            Self::GasStation             => Self::TYPE_GAS_STATION,
            Self::HeatingEndstation      => Self::TYPE_HEATING_ENDSTATION,
            Self::HeatingPlant           => Self::TYPE_HEATING_PLANT,
            Self::HeatingSwitch          => Self::TYPE_HEATING_SWITCH,
            Self::Hospital               => Self::TYPE_HOSPITAL,
            Self::Hotel                  => Self::TYPE_HOTEL,
            Self::Kindergarten           => Self::TYPE_KINDERGARTEN,
            Self::Kino                   => Self::TYPE_KINO,
            Self::Living                 => Self::TYPE_LIVING,
            Self::MineBauxite            => Self::TYPE_MINE_BAUXITE,
            Self::MineCoal               => Self::TYPE_MINE_COAL,
            Self::MineGravel             => Self::TYPE_MINE_GRAVEL,
            Self::MineIron               => Self::TYPE_MINE_IRON,
            Self::MineOil                => Self::TYPE_MINE_OIL,
            Self::MineUranium            => Self::TYPE_MINE_URANIUM,
            Self::MineWood               => Self::TYPE_MINE_WOOD,
            Self::Monument               => Self::TYPE_MONUMENT,
            Self::Parking                => Self::TYPE_PARKING,
            Self::PassangerStation       => Self::TYPE_PASSANGER_STATION,
            Self::PedestrianBridge       => Self::TYPE_PEDESTRIAN_BRIDGE,
            Self::PoliceStation          => Self::TYPE_POLICE_STATION,
            Self::PollutionMeter         => Self::TYPE_POLLUTION_METER,
            Self::Powerplant             => Self::TYPE_POWERPLANT,
            Self::ProductionLine         => Self::TYPE_PRODUCTION_LINE,
            Self::Pub                    => Self::TYPE_PUB,
            Self::RailTrafo              => Self::TYPE_RAIL_TRAFO,
            Self::Raildepo               => Self::TYPE_RAILDEPO,
            Self::Roaddepo               => Self::TYPE_ROADDEPO,
            Self::School                 => Self::TYPE_SCHOOL,
            Self::ShipDock               => Self::TYPE_SHIP_DOCK,
            Self::Shop                   => Self::TYPE_SHOP,
            Self::Sport                  => Self::TYPE_SPORT,
            Self::Storage                => Self::TYPE_STORAGE,
            Self::Substation             => Self::TYPE_SUBSTATION,
            Self::Transformator          => Self::TYPE_TRANSFORMATOR,
            Self::University             => Self::TYPE_UNIVERSITY,
        };

        write!(f, "{}", s)
    }
}


impl Display for BuildingSubtype {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Aircustom        => Self::SUBTYPE_AIRCUSTOM,
            Self::Airplane         => Self::SUBTYPE_AIRPLANE,
            Self::Cableway         => Self::SUBTYPE_CABLEWAY,
            Self::Hostel           => Self::SUBTYPE_HOSTEL,
            Self::Medical          => Self::SUBTYPE_MEDICAL,
            Self::Radio            => Self::SUBTYPE_RADIO,
            Self::Rail             => Self::SUBTYPE_RAIL,
            Self::Restaurant       => Self::SUBTYPE_RESTAURANT,
            Self::Road             => Self::SUBTYPE_ROAD,
            Self::Ship             => Self::SUBTYPE_SHIP,
            Self::Soviet           => Self::SUBTYPE_SOVIET,
            Self::SpaceForVehicles => Self::SUBTYPE_SPACE_FOR_VEHICLES,
            Self::Technical        => Self::SUBTYPE_TECHNICAL,
            Self::Television       => Self::SUBTYPE_TELEVISION,
            Self::Trolleybus       => Self::SUBTYPE_TROLLEYBUS,
        };

        write!(f, "{}", s)
    }
}


impl Display for super::Connection2PType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::AirRoad         => Self::CONN_AIRROAD,
            Self::Pedestrian      => Self::CONN_PEDESTRIAN,
            Self::Road            => Self::CONN_ROAD,
            Self::RoadAllowpass   => Self::CONN_ROAD_ALLOWPASS,
            Self::RoadBorder      => Self::CONN_ROAD_BORDER,
            Self::RoadIn          => Self::CONN_ROAD_IN,
            Self::RoadOut         => Self::CONN_ROAD_OUT,
            Self::Rail            => Self::CONN_RAIL,
            Self::RailAllowpass   => Self::CONN_RAIL_ALLOWPASS,
            Self::RailBorder      => Self::CONN_RAIL_BORDER,
            Self::HeatingBig      => Self::CONN_HEATING_BIG,
            Self::HeatingSmall    => Self::CONN_HEATING_SMALL,
            Self::SteamIn         => Self::CONN_STEAM_IN,
            Self::SteamOut        => Self::CONN_STEAM_OUT,
            Self::PipeIn          => Self::CONN_PIPE_IN,
            Self::PipeOut         => Self::CONN_PIPE_OUT,
            Self::BulkIn          => Self::CONN_BULK_IN,
            Self::BulkOut         => Self::CONN_BULK_OUT,
            Self::Cableway        => Self::CONN_CABLEWAY,
            Self::Factory         => Self::CONN_FACTORY,
            Self::ConveyorIn      => Self::CONN_CONVEYOR_IN,
            Self::ConveyorOut     => Self::CONN_CONVEYOR_OUT,
            Self::ElectricHighIn  => Self::CONN_ELECTRIC_H_IN,
            Self::ElectricHighOut => Self::CONN_ELECTRIC_H_OUT,
            Self::ElectricLowIn   => Self::CONN_ELECTRIC_L_IN,
            Self::ElectricLowOut  => Self::CONN_ELECTRIC_L_OUT,
        };

        write!(f, "{}", s)
    }
}


impl Display for super::StorageCargoType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Passanger => Self::PASSANGER,
            Self::Cement    => Self::CEMENT,
            Self::Covered   => Self::COVERED,
            Self::Gravel    => Self::GRAVEL,
            Self::Oil       => Self::OIL,
            Self::Open      => Self::OPEN,
            Self::Cooler    => Self::COOLER,
            Self::Concrete  => Self::CONCRETE,
            Self::Livestock => Self::LIVESTOCK,
            Self::General   => Self::GENERAL
        };

        write!(f, "{}", s)
    }
}


impl Display for super::ConstructionAutoCost {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Ground        => Self::GROUND,
            Self::GroundAsphalt => Self::GROUND_ASPHALT,
            Self::WallConcrete  => Self::WALL_CONCRETE,
            Self::WallPanels    => Self::WALL_PANELS,
            Self::WallBrick     => Self::WALL_BRICK,
            Self::WallSteel     => Self::WALL_STEEL,
            Self::WallWood      => Self::WALL_WOOD,
            Self::TechSteel     => Self::TECH_STEEL,
            Self::ElectroSteel  => Self::ELECTRO_STEEL,
            Self::RoofWoodBrick => Self::ROOF_WOOD_BRICK,
            Self::RoofSteel     => Self::ROOF_STEEL,
            Self::RoofWoodSteel => Self::ROOF_WOOD_STEEL,
        };

        write!(f, "{}", s)
    }
}


impl Display for super::ConstructionPhase {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Groundworks     => Self::GROUNDWORKS,
            Self::BoardsLaying    => Self::BOARDS_LAYING,
            Self::BricksLaying    => Self::BRICKS_LAYING,
            Self::SkeletonCasting => Self::SKELETON_CASTING,
            Self::SteelLaying     => Self::STEEL_LAYING,
            Self::PanelsLaying    => Self::PANELS_LAYING,
            Self::RooftopBuilding => Self::ROOFTOP_BUILDING,
            Self::WireLaying      => Self::WIRE_LAYING,
            Self::Tunneling       => Self::TUNNELING,
        };

        write!(f, "{}", s)
    }
}


impl Display for super::ResourceType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::Alcohol           => Self::ALCOHOL,
            Self::Alumina           => Self::ALUMINA,
            Self::Aluminium         => Self::ALUMINIUM,
            Self::Asphalt           => Self::ASPHALT,
            Self::Bauxite           => Self::BAUXITE,
            Self::Bitumen           => Self::BITUMEN,
            Self::Boards            => Self::BOARDS,
            Self::Bricks            => Self::BRICKS,
            Self::Cement            => Self::CEMENT,
            Self::Chemicals         => Self::CHEMICALS,
            Self::Clothes           => Self::CLOTHES,
            Self::Coal              => Self::COAL,
            Self::Concrete          => Self::CONCRETE,
            Self::Crops             => Self::CROPS,
            Self::ElectroComponents => Self::ELECTRO_COMP,
            Self::Electricity       => Self::ELECTRICITY,
            Self::Electronics       => Self::ELECTRONICS,
            Self::Fabric            => Self::FABRIC,
            Self::Food              => Self::FOOD,
            Self::Fuel              => Self::FUEL,
            Self::Gravel            => Self::GRAVEL,
            Self::Iron              => Self::IRON,
            Self::Livestock         => Self::LIVESTOCK,
            Self::MechComponents    => Self::MECH_COMP,
            Self::Meat              => Self::MEAT,
            Self::NuclearFuel       => Self::NUCLEAR_FUEL,
            Self::NuclearWaste      => Self::NUCLEAR_WASTE,
            Self::Oil               => Self::OIL,
            Self::Plastic           => Self::PLASTIC,
            Self::PrefabPanels      => Self::PREFABS,
            Self::RawBauxite        => Self::RAW_BAUXITE,
            Self::RawCoal           => Self::RAW_COAL,
            Self::RawIron           => Self::RAW_IRON,
            Self::Steel             => Self::STEEL,
            Self::UF6               => Self::UF_6,
            Self::Uranium           => Self::URANIUM,
            Self::Vehicles          => Self::VEHICLES,
            Self::Wood              => Self::WOOD,
            Self::Workers           => Self::WORKERS,
            Self::Yellowcake        => Self::YELLOWCAKE,
        };

        write!(f, "{}", s)
    }
}


impl Display for super::ParticleType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::ResidentialHeating => Self::RESIDENTIAL_HEATING,
            Self::BigBlack    => Self::FACTORY_BIG_BLACK,
            Self::MediumBlack => Self::FACTORY_MEDIUM_BLACK,
            Self::SmallBlack  => Self::FACTORY_SMALL_BLACK,
            Self::BigGray     => Self::FACTORY_BIG_GRAY,
            Self::MediumGray  => Self::FACTORY_MEDIUM_GRAY,
            Self::SmallGray   => Self::FACTORY_SMALL_GRAY,
            Self::BigWhite    => Self::FACTORY_BIG_WHITE,
            Self::MediumWhite => Self::FACTORY_MEDIUM_WHITE,
            Self::SmallWhite  => Self::FACTORY_SMALL_WHITE,
        };

        write!(f, "{}", s)
    }
}


impl Display for super::AirplaneStationType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s = match self {
            Self::M30 => Self::AIRPLANE_STATION_30M,
            Self::M40 => Self::AIRPLANE_STATION_40M,
            Self::M50 => Self::AIRPLANE_STATION_50M,
            Self::M75 => Self::AIRPLANE_STATION_75M,
        };

        write!(f, "{}", s)
    }
}


impl Display for Point3f {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl<T: Display> Display for Tagged2Points<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} {} {}", self.tag, self.p1, self.p2)
    }
}

impl Display for super::Rect {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "({}, {}, {}, {})", self.x1, self.z1, self.x2, self.z2)
    }
}

impl Display for super::QuotedStringParam<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Self(s) = self;
        write!(f, "\"{}\"", s)
    }
}

impl Display for super::IdStringParam<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let Self(s) = self;
        write!(f, "{}", s)
    }
}

impl Display for super::StrValue<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let s: &str = match self {
            Self::Borrowed(s) => s,
            Self::Owned(s) => s.as_str()
        };

        write!(f, "{}", s)
    }
}
