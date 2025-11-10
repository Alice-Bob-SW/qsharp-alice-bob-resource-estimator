import numpy as np

validity_range_4k = ['Cu_RRR_100', 'PTFE']
validity_range_2k = ['NbTi', 'CN', 'SCN', 'BeCu','PhBr']

class Material:
    material_id: str
    fit_params: np.array

    def __init__(self, material_id, fit_params):
        self.material_id = material_id
        self.fit_params = fit_params
        
    def k(self,T):
        if T < 4 and self.material_id in validity_range_4k:
            return T * self.k(4) / 4
        elif T < 2 and self.material_id in validity_range_2k:
            return T * self.k(2) / 2
        elif self.material_id == 'Cu_RRR_100':
            NUM = self.fit_params[0] + self.fit_params[2]*(T**(0.5)) + self.fit_params[4]*(T**(1.0)) + self.fit_params[6]*(T**(1.5)) + self.fit_params[8]*(T**(2.0))
            DENOM = 1 + self.fit_params[1]*(T**(0.5)) + self.fit_params[3]*(T**(1.0)) + self.fit_params[5]*(T**(1.5)) + self.fit_params[7]*(T**(2.0))
            return 10**(NUM / DENOM)
        else:
            p = np.poly1d(self.fit_params)
            return 10**p(np.log10(T))

class Materials_DB:
    db: dict[str, Material] = {}

    def __init__(self, materials: list[Material]):
        for material in materials:
            self.db[material.material_id] = material

NbTi = Material(
    material_id = 'NbTi',
    fit_params = np.array([ 0.02612193, -0.18559454,  0.52131209, -1.01746415,  2.15843516, -3.4815713 ,  2.63612161,  0.67880346, -1.58508579])
)

CN = Material(
    material_id = 'CN',
    fit_params = np.array([-0.527, 5.985, -28.749, 76.215, -121.477, 117.69, -66.114, 20.499, -3.198])
)

PTFE = Material(
    material_id = 'PTFE',
    fit_params = np.array([0.33829, -4.3135, 23.32, -69.556, 124.69, -136.99, 89.43, -30.677, 2.738])
)

SCN = Material(
    material_id = 'SCN',
    fit_params = np.array([-0.025, 1.322, -11.825, 46.383, -96.844, 113.586, -74.184, 25.845, -2.750])
)

BeCu = Material(
    material_id = 'BeCu',
    fit_params = np.array([0, -0.10501, 0.68722, -1.6145, 1.2788, 0.71218, -1.6954, 1.9319, -0.50015])
)

PhBr = Material(
    material_id = 'PhBr',
    fit_params = np.array([-0.01754878, 0.0701625, 0.05263581, -0.35246336, -0.50597204, 2.50133348, -2.76954821, 2.34173576, -0.65757732])
)

Cu_RRR_100 = Material(
    material_id = 'Cu_RRR_100',
    fit_params = np.array([2.2154, -0.47461, -0.88068, 0.13871, 0.29505, -0.02043, -0.04831, 0.001281, 0.003207])
)

materials = Materials_DB([NbTi, CN, PTFE, SCN, BeCu, Cu_RRR_100, PhBr])