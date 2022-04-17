# [0.2.0](https://github.com/magnesium-uploader/oxide/compare/v0.1.6...v0.2.0) (2022-04-17)


### Bug Fixes

* :bug: Use i64 to support bigger byte sizes ([2f3e5ee](https://github.com/magnesium-uploader/oxide/commit/2f3e5ee8a68cf2599011cf5615ef837104858063))
* :technologist: Use GENERICS for logging! ([de86457](https://github.com/magnesium-uploader/oxide/commit/de86457a3aa538d30a14ce95155502f792a53912))


### Features

* :boom: Completely overhaul the privileges system to use Bitwise enums instead of string arrays! ([5fa6897](https://github.com/magnesium-uploader/oxide/commit/5fa6897e57f263398444903e663e791baa0152a1))



## [0.1.6](https://github.com/magnesium-uploader/oxide/compare/v0.1.5...v0.1.6) (2022-03-27)



## [0.1.5](https://github.com/magnesium-uploader/oxide/compare/v0.1.4...v0.1.5) (2022-03-27)



## [0.1.4](https://github.com/magnesium-uploader/oxide/compare/v0.1.3...v0.1.4) (2022-03-18)


### Bug Fixes

* **/api/v1/files:** :bug: return unauthorized on no auth header ([fd1b2ad](https://github.com/magnesium-uploader/oxide/commit/fd1b2ad21c29a8798dc40fcbe8457269b621e581))



## [0.1.3](https://github.com/magnesium-uploader/oxide/compare/v0.1.2...v0.1.3) (2022-03-18)



## [0.1.2](https://github.com/magnesium-uploader/oxide/compare/v0.1.1...v0.1.2) (2022-03-18)


### Bug Fixes

* :bug: release_tag and tag_name are invalid, fixed ([8d7e060](https://github.com/magnesium-uploader/oxide/commit/8d7e060cf47b50642f4e807604b1f248ed69573f))



## [0.1.1](https://github.com/magnesium-uploader/oxide/compare/v0.1.0...v0.1.1) (2022-03-18)


### Bug Fixes

* :green_heart: Use actions-rs instead of running commands raw ([fcc6fc1](https://github.com/magnesium-uploader/oxide/commit/fcc6fc1069c9e63cc0b374e1efeba36c86de27e7))



# [0.1.0](https://github.com/magnesium-uploader/oxide/compare/43ef4e1c63df6fa0f4e9b76df07eabb295d22697...v0.1.0) (2022-03-18)


### Bug Fixes

* :green_heart: Rename rust-prod -> rust.yml and delete rust-dev ([c6fa9fb](https://github.com/magnesium-uploader/oxide/commit/c6fa9fb458c7b494d9a792f12b58205433ea2a73))
* **/api/v1/files:** :ambulance: delete file extention fix ([e64b7c5](https://github.com/magnesium-uploader/oxide/commit/e64b7c54d9a64ee26baa8e0edf0a6f2006fcc051))
* **/api/v1/files:** :ambulance: Save as .mgo (missed in refractor) ([ac5144a](https://github.com/magnesium-uploader/oxide/commit/ac5144aa26d352621ce58804d11670a4bb23afc9))
* **/api/v1/files:** :bug: Check if the user has permission to upload ([43ef4e1](https://github.com/magnesium-uploader/oxide/commit/43ef4e1c63df6fa0f4e9b76df07eabb295d22697))


### Features

* :beers: "Encryption" ([d0364aa](https://github.com/magnesium-uploader/oxide/commit/d0364aa8516685f1031e7c6b6c5e88ecb25a957a))
* :sparkles: A hopefully working CI & CD ([2e25254](https://github.com/magnesium-uploader/oxide/commit/2e25254eaff249a3dad53a6720c68e5b8eaa0d2e))
* :sparkles: BREAKING CHANGES: error handeling ([f31a541](https://github.com/magnesium-uploader/oxide/commit/f31a54167623360ae5c4daa51aedef93068c5491))
* **/api/v1/files:** :sparkles: Add Content-Disposition to return the real filename ([a82f87e](https://github.com/magnesium-uploader/oxide/commit/a82f87e01b0fc85b4d6eedd3e4acb0cb6478d135))
* **/api/v1/files:** :sparkles: working zws implementation ([2b13b59](https://github.com/magnesium-uploader/oxide/commit/2b13b5988206bb1de03c1705f892f10cc14814be))



