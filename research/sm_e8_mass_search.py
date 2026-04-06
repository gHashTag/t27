#!/usr/bin/env python3
"""SM mass ratios vs E8 Zamolodchikov numbers - search script"""
import math
PHI = (1 + math.sqrt(5)) / 2
PI = math.pi
zm = [1.0, 2*math.cos(PI/5), 2*math.cos(PI/30),
      4*math.cos(PI/5)*math.cos(7*PI/30),
      4*math.cos(PI/5)*math.cos(2*PI/15),
      4*math.cos(PI/5)*math.cos(PI/30),
      8*math.cos(PI/5)**2*math.cos(7*PI/30),
      8*math.cos(PI/5)**2*math.cos(2*PI/15)]

sm = {'e': 0.511, 'mu': 105.658, 'tau': 1776.86,
      'u': 2.16, 'd': 4.67, 's': 93.4,
      'c': 1270, 'b': 4180, 't': 172760,
      'W': 80369, 'Z': 91188, 'H': 125250}

print("KEY FINDING: m_u/m_e vs phi^3")
print(f"  m_u/m_e = {sm['u']/sm['e']:.6f}")
print(f"  phi^3   = {PHI**3:.6f}")
print(f"  Error:    {abs(sm['u']/sm['e'] - PHI**3)/(sm['u']/sm['e'])*100:.3f}%")
