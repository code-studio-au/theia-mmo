Altered local copy of Landmass 0.9.3 from crates.io. Original licenses retained.
Crate checksum: fd2fd988b81194c6f02c293a28b510b44f1d8581c80b1bfe498e85e9c3f57b93.

Theia modification, 22 November 2026: expose the existing 0.01 m island-edge
link distance as ArchipelagoOptions.edge_link_distance; retain 0.01 by default
and require a finite positive value. No linking algorithm changes.
The previous hardcoded tolerance disconnected adjoining Millhaven tiles whose
0.05 m vertical rasterization phases differ. Theia's static query chooses at
most two height cells, capped at 0.1 m and the admitted step height (with the
original 0.01 m minimum). Strict floor sampling, obstacle erosion and independent
collision/clearance checks remain in force. No agents or crowd avoidance used.
