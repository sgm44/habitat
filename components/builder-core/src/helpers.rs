// Copyright (c) 2016 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use protocol::originsrv::OriginPackageVisibility;

pub fn transition_visibility(
    incoming: OriginPackageVisibility,
    existing: OriginPackageVisibility,
) -> OriginPackageVisibility {
    match (existing, incoming) {
        (OriginPackageVisibility::Private, OriginPackageVisibility::Hidden) => {
            OriginPackageVisibility::Private
        }
        (OriginPackageVisibility::Private, OriginPackageVisibility::Private) => {
            OriginPackageVisibility::Private
        }
        (_, OriginPackageVisibility::Private) => OriginPackageVisibility::Hidden,
        (_, OriginPackageVisibility::Hidden) => OriginPackageVisibility::Hidden,
        (_, OriginPackageVisibility::Public) => OriginPackageVisibility::Public,
    }
}
