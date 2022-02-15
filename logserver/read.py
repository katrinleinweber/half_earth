import sys
from db import Database
from datetime import datetime
import matplotlib.pyplot as plt

if len(sys.argv) > 1:
    TARGET_SESSION = sys.argv[1]
else:
    TARGET_SESSION = None

def gtco2eq(byproducts):
  co2eq = byproducts['co2_emissions'] + byproducts['ch4_emissions'] * 36 + byproducts['n2o_emissions'] * 298
  return co2eq * 1e-15

def pprint(obj, indent=0, prefix=''):
    if isinstance(obj, list):
        for o in obj:
            pprint(o, indent=indent+1, prefix='- ')
    elif isinstance(obj, dict):
        for i, (k, v) in enumerate(obj.items()):
            if isinstance(v, dict):
                pprint(v, indent=indent+1)
            else:
                print('{}{}{}: {}'.format(
                    prefix if i == 0 else '',
                    '  '*(indent-1 if i == 0 else indent),
                    k, v))

def emissions(snapshots):
    return [{
        'year': s['snapshot']['gameState']['world']['year'],
        'emissions': gtco2eq(s['snapshot']['gameState']['world'])
    } for s in snapshots]

def process_mix(snapshots):
    return [{
        'name': p['name'],
        'mix_share': p['mix_share']
    } for p in snapshots[-1]['snapshot']['gameState']['processes']]

def electricity_demand(snapshots):
    return [{
        'year': s['snapshot']['gameState']['world']['year'],
        'demand': s['snapshot']['gameState']['output_demand']['electricity'] * 1e-9
    } for s in snapshots]

def active_projects(snapshots):
    projects = snapshots[-1]['snapshot']['gameState']['projects']
    return [p for p in projects if p['status'] != 'Inactive']

def project_timeline(snapshots):
    timeline = []
    last_projects = {}
    for i, s in enumerate(snapshots):
        year = s['snapshot']['gameState']['world']['year']
        projects = s['snapshot']['gameState']['projects']
        timeline.append({
            'year': year
        })
        for p in projects:
            if p['status'] != 'Inactive':
                last = last_projects.get(p['name'])
                if last is None or p['status'] != last:
                    timeline[-1][p['name']] = p['status']
                last_projects[p['name']] = p['status']
    return timeline

def events(snapshots):
    return [{
        'year': s['snapshot']['gameState']['world']['year'],
        'events': s['snapshot']['events']
    } for s in snapshots]


if __name__ == '__main__':
    db = Database('logs.db')

    print('Recent Sessions:')
    print('-'*50)
    sessions = db.sessions()
    sessions.reverse()
    for session in sessions[:10]:
        print(session['id'])
        print(' ', datetime.fromtimestamp(float(session['timestamp'])))
        print('  Version:', session['version'])
        print('  User-Agent:', session['useragent'])
        print('  Snapshots:', len(db.snapshots(session['id'])))
        snapshots = db.snapshots(session['id'])
        if session['id'] == TARGET_SESSION:
            snapshots = db.snapshots(session['id'])
            import ipdb; ipdb.set_trace()