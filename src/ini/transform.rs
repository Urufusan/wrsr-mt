use crate::ini;
use crate::ini::common::{Point3f, Rect};
use crate::ini::BuildingToken;


pub fn scale_building(file: &mut ini::BuildingIni<'_>, factor: f64) {
    let mul = |x: f32| { ((x as f64) * factor) as f32 };

    for (_, t_state) in file.tokens.iter_mut() {
        t_state.modify(|t_source|
            transform_point(t_source, |p| p.scaled(factor))
            .or_else(|| transform_rect(t_source, |r| Rect { x1: mul(r.x1), 
                                                            x2: mul(r.x2), 
                                                            z1: mul(r.z1), 
                                                            z2: mul(r.z2) }))
            .or_else(|| { 
                use crate::ini::BuildingToken as T;
                use crate::ini::building::ResourceVisualization as RV;
                match t_source {
                    T::HeliportArea(x)              => Some(T::HeliportArea(mul(*x))),
                    T::HarborTerrainFrom(x)         => Some(T::HarborTerrainFrom(mul(*x))),
                    T::HarborWaterFrom(x)           => Some(T::HarborWaterFrom(mul(*x))),
                    T::HarborExtendWhenBuilding(x)  => Some(T::HarborExtendWhenBuilding(mul(*x))),

                    T::ResourceVisualization(rv) => Some(T::ResourceVisualization (RV {
                        storage_id: rv.storage_id,
                        position:   rv.position.scaled(factor),
                        rotation:   rv.rotation,
                        scale:      rv.scale.scaled(factor),
                        numstep_x:  (mul(rv.numstep_x.0), rv.numstep_x.1),
                        numstep_z:  (mul(rv.numstep_z.0), rv.numstep_z.1),
                    })),
                    _ => None 
                }})
        )
    }
}


pub fn scale_render(f: &mut ini::RenderIni<'_>, factor: f64) {
    use crate::ini::RenderToken as T;

    for (_, t_state) in f.tokens.iter_mut() {
        t_state.modify(|t| match t {
           T::Light((pt, x))            => Some(T::Light((pt.scaled(factor), *x))),
           T::LightRgb((pt, x, c))      => Some(T::LightRgb((pt.scaled(factor), *x, *c))),
           T::LightRgbBlink((pt, x, c)) => Some(T::LightRgbBlink((pt.scaled(factor), *x, *c))),
            _ => None 
        });
    }
}


//----------------------------------------------------------------------------------------------


fn transform_point<'a, F: Fn(&Point3f) -> Point3f>(t: &BuildingToken<'a>, f: F) -> Option<BuildingToken<'a>> {
    use crate::ini::BuildingToken as T;
    match t {
        T::VehicleStation((p1, p2))               => Some(T::VehicleStation((                f(p1), f(p2)  ))),
        T::VehicleStationDetourPoint(p1)          => Some(T::VehicleStationDetourPoint(      f(p1)          )),
        T::VehicleStationDetourPid((i, p1))       => Some(T::VehicleStationDetourPid((  *i,  f(p1)         ))),
        T::VehicleParking((p1, p2))               => Some(T::VehicleParking((                f(p1), f(p2)  ))),
        T::VehicleParkingDetourPoint(p1)          => Some(T::VehicleParkingDetourPoint(      f(p1)          )),
        T::VehicleParkingDetourPid((i, p1))       => Some(T::VehicleParkingDetourPid((  *i,  f(p1)         ))),
        T::VehicleParkingPersonal((p1, p2))       => Some(T::VehicleParkingPersonal((        f(p1), f(p2)  ))),

        T::AirplaneStation((t, p1, p2))           => Some(T::AirplaneStation((          *t,  f(p1), f(p2)  ))),
        T::HeliportStation((p1, p2))              => Some(T::HeliportStation((               f(p1), f(p2)  ))),
        T::ShipStation((p1, p2))                  => Some(T::ShipStation((                   f(p1), f(p2)  ))),

        T::Connection2Points((t, p1, p2))         => Some(T::Connection2Points((        *t,  f(p1), f(p2)  ))),
        T::Connection1Point((t, p1))              => Some(T::Connection1Point((         *t,  f(p1)         ))),
        T::OffsetConnection((i, p1))              => Some(T::OffsetConnection((         *i,  f(p1)         ))),

        T::Particle((t, p1, x, y))                => Some(T::Particle((                  *t, f(p1), *x, *y ))),
        T::ParticleReactor(p1)                    => Some(T::ParticleReactor(                f(p1)          )),

        T::TextCaption((p1, p2))                  => Some(T::TextCaption((                   f(p1), f(p2)  ))),
        T::WorkerRenderingArea((p1, p2))          => Some(T::WorkerRenderingArea((           f(p1), f(p2)  ))),
        T::ResourceIncreasePoint((i, p1))         => Some(T::ResourceIncreasePoint((    *i,  f(p1)         ))),
        T::ResourceIncreaseConvPoint((i, p1, p2)) => Some(T::ResourceIncreaseConvPoint((*i,  f(p1), f(p2)  ))),
        T::ResourceFillingPoint(p1)               => Some(T::ResourceFillingPoint(           f(p1)          )),
        T::ResourceFillingConvPoint((p1, p2))     => Some(T::ResourceFillingConvPoint((      f(p1), f(p2)  ))),

        T::CostWorkVehicleStation((p1, p2))       => Some(T::CostWorkVehicleStation((        f(p1), f(p2)  ))),

        _ => None 
    }
}


fn transform_rect<'a, F: Fn(&Rect) -> Rect>(t: &BuildingToken<'a>, f: F) -> Option<BuildingToken<'a>> {
    use crate::ini::BuildingToken as T;
    match t {
        T::ConnectionsSpace(r)                 => Some(T::ConnectionsSpace(f(r))),
        T::ConnectionsRoadDeadSquare(r)        => Some(T::ConnectionsRoadDeadSquare(f(r))),
        T::ConnectionsWaterDeadSquare((x, r))  => Some(T::ConnectionsWaterDeadSquare((*x, f(r)))),
        _ => None 
    }
}

