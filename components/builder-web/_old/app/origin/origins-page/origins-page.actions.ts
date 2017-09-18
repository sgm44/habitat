import { addNotification, SUCCESS, DANGER } from "../../actions/notifications";
import { BuilderApiClient } from "../../BuilderApiClient";

export const POPULATE_MY_ORIGIN_INVITATIONS = "POPULATE_MY_ORIGIN_INVITATIONS";

export function acceptOriginInvitation(invitationId: string, originName: string, token: string) {
  return dispatch => {
      new BuilderApiClient(token).acceptOriginInvitation(invitationId, originName).
          then(response => {
              dispatch(addNotification({
                  title: "Invitation Accepted",
                  body: "You are now a member",
                  type: SUCCESS,
              }));
              dispatch(fetchMyOriginInvitations(token));
          }).catch(error => {
              dispatch(addNotification({
                  title: "Invitation Acceptance Failed",
                  body: error.message,
                  type: DANGER,
              }));
          });
  };
}

export function fetchMyOriginInvitations(token) {
  return dispatch => {
      new BuilderApiClient(token).getMyOriginInvitations().
          then(invitations => {
              dispatch(populateMyOriginInvitations(invitations));
          }).catch(error => {
              dispatch(populateMyOriginInvitations(undefined, error));
          });
  };
}

function populateMyOriginInvitations(payload, error = undefined) {
  return {
      type: POPULATE_MY_ORIGIN_INVITATIONS,
      payload,
      error,
  };
}
