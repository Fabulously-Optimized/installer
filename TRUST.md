# Procedure Overview
1. Frontend calls Modrinth API to retrieve information about current modpack versions
1. User selects version and other install parameters
1. Frontend sends modpack bundle URL alongside verification material to backend
1. Backend downloads modpack bundle and verifies it
1. Backend removes any previously installed files
1. Backend downloads mods as specified by manifest and verifies their SHA512 hashes
1. Backend installs misc. files contained in bundle
1. Backend queries Fabric Meta to install Fabric version requested by manifest
1. Backend creates launcher profile and writes install metadata to file

# Trust & Security
- Modpack bundle is built from [source](https://github.com/Fabulously-Optimized/fabulously-optimized) and signed by Github Actions using cosign
- Source and modpack bundle includes hashes for all their required external files
- Backend restricts download URLs for modpack manifest and external mods
- Backend calls Verifier to verify modpack signature
- Verifier verifies modpack was built from source using Github Actions, as part of a release pipeline