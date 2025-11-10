import numpy as np
import materials as mat

class Layer:
    material: mat.Material
    diameter: float
    
    def __init__(self, material, diameter):
        self.material = material
        self.diameter = diameter

class Cable:
    cable_ref: str
    outer: Layer
    dielectric: Layer
    inner: Layer

    def __init__(self, cable_ref, outer, dielectric, inner):
        self.cable_ref = cable_ref
        self.outer = outer
        self.dielectric = dielectric
        self.inner = inner

    @property
    def outer_area(self):
        R1 = self.outer.diameter / 2
        R2 = self.dielectric.diameter / 2

        return np.pi*(R1**2 - R2**2)

    @property
    def dielectric_area(self):
        R1 = self.dielectric.diameter / 2
        R2 = self.inner.diameter / 2

        return np.pi*(R1**2 - R2**2)

    @property
    def inner_area(self):
        R1 = self.inner.diameter / 2

        return np.pi*(R1**2)

class Cables_DB:
    db: dict[str, Cable] = {}

    def __init__(self, cables: list[Cable]):
        for cable in cables:
            self.db[cable.cable_ref] = cable

Radiall_NbTi_cable = Cable(
    cable_ref = 'Radiall NbTi',
    outer=Layer(
        material=mat.materials.db['NbTi'],
        diameter=0.9e-3,
    ),
    dielectric=Layer(
        material=mat.materials.db['PTFE'],
        diameter=0.66e-3,
    ),
    inner=Layer(
        material=mat.materials.db['NbTi'],
        diameter=0.203e-3,
    ),
)

SC_119_50_SCN_CN = Cable(
    cable_ref = 'SC-119/50-SCN-CN',
    outer=Layer(
        material=mat.materials.db['SCN'],
        diameter=1.19e-3,
    ),
    dielectric=Layer(
        material=mat.materials.db['PTFE'],
        diameter=0.94e-3,
    ),
    inner=Layer(
        material=mat.materials.db['CN'],
        diameter=0.287e-3,
    ),
)

SC_86_50_SCN_CN = Cable(
    cable_ref = 'SC-86/50-SCN-CN',
    outer=Layer(
        material=mat.materials.db['SCN'],
        diameter=0.86e-3,
    ),
    dielectric=Layer(
        material=mat.materials.db['PTFE'],
        diameter=0.66e-3,
    ),
    inner=Layer(
        material=mat.materials.db['CN'],
        diameter=0.203e-3,
    ),
)

SC_86_50_NbTi_NbTi = Cable(
    cable_ref = 'SC-86/50-NbTi-NbTi',
    outer=Layer(
        material=mat.materials.db['NbTi'],
        diameter=0.90e-3,
    ),
    dielectric=Layer(
        material=mat.materials.db['PTFE'],
        diameter=0.66e-3,
    ),
    inner=Layer(
        material=mat.materials.db['NbTi'],
        diameter=0.203e-3,
    ),
)

PhBr_36AWG = Cable(
    cable_ref= 'PhBr_36AWG',
    outer = Layer(material=mat.materials.db['PhBr'], diameter = 0.127e-3,),
    dielectric = Layer(material=mat.materials.db['PTFE'], diameter = 0.127e-3,),
    inner = Layer(material=mat.materials.db['PhBr'], diameter = 0.127e-3,)
)

NbTi_36AWG = Cable(
    cable_ref= 'NbTi_36AWG',
    outer = Layer(material=mat.materials.db['NbTi'], diameter = 0.127e-3,),
    dielectric = Layer(material=mat.materials.db['NbTi'], diameter = 0.127e-3,),
    inner = Layer(material=mat.materials.db['NbTi'], diameter = 0.127e-3,)
)

cables = Cables_DB([Radiall_NbTi_cable, SC_119_50_SCN_CN, SC_86_50_SCN_CN,
                    SC_86_50_NbTi_NbTi, PhBr_36AWG, NbTi_36AWG])