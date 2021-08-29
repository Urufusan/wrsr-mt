use crate::ini;
use crate::ini::common::{Point3f, Rect};
use crate::ini::BuildingToken;


pub fn scale_building(file: &mut ini::BuildingIni<'_>, factor: f64) {
    let mul = |x: f32| { ((x as f64) * factor) as f32 };

    for (_, t_state) in file.tokens.iter_mut() {
        t_state.modify(|t_source| {
                use crate::ini::BuildingToken as T;
                use crate::ini::building::ResourceVisualization as RV;
                match t_source {
                    T::HeliportArea(x)               => Some(T::HeliportArea(mul(*x))),
                    T::HarborTerrainFrom(x)          => Some(T::HarborTerrainFrom(mul(*x))),
                    T::HarborWaterFrom(x)            => Some(T::HarborWaterFrom(mul(*x))),
                    T::HarborExtendWhenBuilding(x)   => Some(T::HarborExtendWhenBuilding(mul(*x))),
                    T::ParticleSnowRemove((p, i, r)) => Some(T::ParticleSnowRemove((p.scaled(factor), *i, mul(*r)))),

                    T::ResourceVisualization(rv) => Some(T::ResourceVisualization (RV {
                        storage_id: rv.storage_id,
                        position:   rv.position.scaled(factor),
                        rotation:   rv.rotation,
                        scale:      rv.scale.scaled(factor),
                        numstep_x:  (mul(rv.numstep_x.0), rv.numstep_x.1),
                        numstep_z:  (mul(rv.numstep_z.0), rv.numstep_z.1),
                    })),
                    other => transform_point(other, |p| p.scaled(factor))
                                 .or_else(|| transform_rect(t_source, |r| Rect { x1: mul(r.x1), 
                                                                                 x2: mul(r.x2), 
                                                                                 z1: mul(r.z1), 
                                                                                 z2: mul(r.z2) }))
                }
            })
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


//-------------------------------------------------------------------


pub fn offset_building(file: &mut ini::BuildingIni<'_>, dx: f32, dy: f32, dz: f32) {
    for (_, t_state) in file.tokens.iter_mut() {
        t_state.modify(|t_source| {
                use crate::ini::BuildingToken as T;
                use crate::ini::building::ResourceVisualization as RV;
                match t_source {
                    T::HarborTerrainFrom(x)          => Some(T::HarborTerrainFrom(*x + dx)),
                    T::HarborWaterFrom(x)            => Some(T::HarborWaterFrom(*x + dx)),
                    T::HarborExtendWhenBuilding(x)   => Some(T::HarborExtendWhenBuilding(*x - dx)),
                    T::ParticleSnowRemove((p, i, r)) => Some(T::ParticleSnowRemove((p.offset(dx, dy, dz), *i, *r))),

                    T::ResourceVisualization(rv) => Some(T::ResourceVisualization (RV {
                        storage_id: rv.storage_id,
                        position:   rv.position.offset(dx, dy, dz),
                        rotation:   rv.rotation,
                        scale:      rv.scale.clone(),
                        numstep_x:  rv.numstep_x,
                        numstep_z:  rv.numstep_z,
                    })),
                    other => transform_point(other, |p| p.offset(dx, dy, dz))
                                 .or_else(|| transform_rect(t_source, |r| Rect { x1: r.x1 + dx, 
                                                                                 x2: r.x2 + dx, 
                                                                                 z1: r.z1 + dz, 
                                                                                 z2: r.z2 + dz }))
                }
            })
    }
}

pub fn offset_render(f: &mut ini::RenderIni<'_>, dx: f32, dy: f32, dz: f32) {
    use crate::ini::RenderToken as T;

    for (_, t_state) in f.tokens.iter_mut() {
        t_state.modify(|t| match t {
           T::Light((pt, x))            => Some(T::Light((pt.offset(dx, dy, dz), *x))),
           T::LightRgb((pt, x, c))      => Some(T::LightRgb((pt.offset(dx, dy, dz), *x, *c))),
           T::LightRgbBlink((pt, x, c)) => Some(T::LightRgbBlink((pt.offset(dx, dy, dz), *x, *c))),
            _ => None 
        });
    }
}


//-------------------------------------------------------------------

fn mirror_z_point(pt: &Point3f) -> Point3f {
    Point3f { x: pt.x, y: pt.y, z: 0f32 - pt.z }
}

pub fn mirror_z_building(file: &mut ini::BuildingIni<'_>) {
    use crate::ini::BuildingToken as T;
    use crate::ini::building::ResourceVisualization as RV;

    for (_, t_state) in file.tokens.iter_mut() {
        t_state.modify(|t_source| match t_source {
            T::ResourceVisualization(rv) => Some(T::ResourceVisualization (RV {
                storage_id: rv.storage_id,
                position:   mirror_z_point(&rv.position),
                rotation:   0f32 - rv.rotation,
                scale:      rv.scale.clone(),
                numstep_x:  rv.numstep_x,
                numstep_z:  ((0f32 - rv.numstep_z.0), rv.numstep_z.1),
            })),
            // must flip these points, otherwise the text faces backwards
            T::TextCaption((p1, p2)) => Some(T::TextCaption((mirror_z_point(p2), mirror_z_point(p1)))),
            T::ParticleSnowRemove((p, i, r)) => Some(T::ParticleSnowRemove((mirror_z_point(p), *i, *r))),
            other => transform_point(other, |p| mirror_z_point(p))
                     .or_else(|| transform_rect(t_source, |r|
                        Rect {  x1: r.x1, 
                                z1: 0f32 - r.z1, 
                                x2: r.x2, 
                                z2: 0f32 - r.z2 }))
        });
    }
}

pub fn mirror_z_render(f: &mut ini::RenderIni<'_>) {
    use crate::ini::RenderToken as T;

    for (_, t_state) in f.tokens.iter_mut() {
        t_state.modify(|t| match t {
           T::Light((pt, x))            => Some(T::Light((mirror_z_point(pt), *x))),
           T::LightRgb((pt, x, c))      => Some(T::LightRgb((mirror_z_point(pt), *x, *c))),
           T::LightRgbBlink((pt, x, c)) => Some(T::LightRgbBlink((mirror_z_point(pt), *x, *c))),
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

