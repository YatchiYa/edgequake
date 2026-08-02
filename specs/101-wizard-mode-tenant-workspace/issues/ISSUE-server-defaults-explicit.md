# ISSUE — Server defaults explicit

**Findings**: F-101-02, F-101-03, F-101-08  
**Laws**: LAW-101-2, LAW-101-3  

## Problem

Chip storm on tenant create; defaults not always labeled with `provider/model` × 3; loading flash.

## Fix

`ServerDefaultsCard` + `ModelDefaultsStep`; happy path no provider bar; skeleton while loading.
