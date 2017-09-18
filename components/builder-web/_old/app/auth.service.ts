import { Injectable } from "@angular/core";
import { Observable } from "rxjs/Observable";
import * as config from "./config";
import { Http, URLSearchParams } from "@angular/http";

@Injectable()
export class AuthService {
  gitHubTokenAuthUrl = `${config["habitat_api_url"]}/authenticate`;

  constructor(private http: Http) {}

  get token(): string {
    return localStorage.getItem("habitat-auth-token");
  }

  set token(token: string) {
    localStorage.setItem("habitat-auth-token", token);
  }

  public isAuthenticated(): boolean {
    return this.token !== undefined;
  }

  public authenticate(params): Observable<string> {
      params = new URLSearchParams(params.slice(1));
      console.log(config["habitat_api_url"]);
      return this.http.get(`${this.gitHubTokenAuthUrl}/${params.get("code")}`)
        .map(res => res.json().token);
  }
}
