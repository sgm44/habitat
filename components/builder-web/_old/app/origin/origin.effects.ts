import "rxjs/add/operator/catch";
import "rxjs/add/operator/map";
import "rxjs/add/operator/switchMap";
import "rxjs/add/operator/skip";
import "rxjs/add/operator/takeUntil";
import { Injectable, InjectionToken, Optional, Inject } from "@angular/core";
import { Effect, Actions } from "@ngrx/effects";
import { Action } from "@ngrx/store";
import { Observable } from "rxjs/Observable";
import { Scheduler } from "rxjs/Scheduler";
import { async } from "rxjs/scheduler/async";
import { empty } from "rxjs/observable/empty";
import { of } from "rxjs/observable/of";

import * as origin from "./origin.actions";
import { Origin } from "./origin.model";
import { OriginService } from "./origin.service";

@Injectable()
export class OriginEffects {
    @Effect()
    loadOrigins$: Observable<Action> = this.actions$
        .ofType(origin.LOAD_ORIGINS)
        .switchMap(() => {
            return this.originService
                .getMyOrigins()
                .map((origins: Origin[]) => new origin.LoadOriginsSuccess(origins))
                .catch(err => of(new origin.LoadOriginsFailure(err)));
        });

    @Effect()
    loadOriginPackageCount$: Observable<Action> = this.actions$
        .ofType<origin.PopulatePackageCountForOrigin>()
        .map(action => action.payload)
        .switchMap((originName: string) => {
            return this.originService
                .getStats(originName)
                .map((count) => new origin.PopulatePackageCountForOrigin(count))
                .catch(err => of(new origin.PopulatePackageCountForOrigin("0")));
        });

    constructor(
        private actions$: Actions,
        private originService: OriginService
    ) {}
}

// export function fetchMyOrigins(token) {
//   return dispatch => {
//       new BuilderApiClient(token).getMyOrigins().then(origins => {
//           dispatch(populateMyOrigins(origins));
//           dispatch(fetchOriginsPackageCount(origins));
//       }).catch(error => dispatch(populateMyOrigins(undefined, error)));
//   };
// }

// export function fetchOrigin(originName: string) {
//   return dispatch => {
//       dispatch(setCurrentOriginLoading(true));
//       new BuilderApiClient().getOrigin(originName).then(response => {
//           dispatch(setCurrentOrigin(response));
//       }).catch(error => {
//           dispatch(setCurrentOrigin(undefined, error));
//       });
//   };
// }

// export function fetchOriginInvitations(originName: string, token: string) {
//   return dispatch => {
//       new BuilderApiClient(token).getOriginInvitations(originName).
//           then(response => {
//               dispatch(populateOriginInvitations(response));
//           }).catch(error => {
//               dispatch(populateOriginInvitations(undefined, error));
//           });
//   };
// }

// export function addDockerHubCredentials(credentials, origin: string, token: string) {
//   // return dispatch => {
//   //     new BuilderApiClient(token).addDockerHubCredentials(origin, credentials).
//   //         then(response => {
//   //             dispatch(fetchIntegrations(origin, token));
//   //         }).catch(error => {
//   //             dispatch(setOriginIntegrationSaveErrorMessage(error.message));
//   //         });
//   // };
// }

// function fetchOriginsPackageCount(origins) {
//   return dispatch => {
//       origins.forEach(origin => {
//           depotApi
//               .getStats(origin)
//               .then(response => {
//                   response["origin"] = origin;
//                   dispatch(populatePackageCountForOrigin(response));
//               })
//               .catch(error => {
//                   dispatch(populatePackageCountForOrigin(error.message));
//               });
//       });
//   };
// }
