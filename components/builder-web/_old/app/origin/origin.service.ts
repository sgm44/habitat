import "rxjs/add/operator/map";
import { Injectable } from "@angular/core";
import { Http } from "@angular/http";
import { Observable } from "rxjs/Observable";
import { Origin } from "./origin.model";
import config from "../config";

@Injectable()
export class OriginService {
    private API_PATH: string = config["habitat_api_url"];

    constructor(private http: Http) {}

    getMyOrigins(): Observable<Origin[]> {
        return this.http.get(`${this.API_PATH}/user/origins`)
            .map(res => res.json() || []);
    }

    getOrigin(originName: string): Observable<Origin> {
        return this.http.get(`${this.API_PATH}/depot/origins/${originName}`)
            .map(res => res.json());
    }

    getOriginInvitations(originName: string): Observable<string[]> {
        return this.http.get(`${this.API_PATH}/depot/origins/${originName}/invitations`)
            .map(res => res.json());
    }

    getStats(originName: string): Observable<any> {
        return this.http.get(`${this.API_PATH}/depot/pkgs/origins/${originName}/stats`)
            .map(res => res.json());
    }
}
