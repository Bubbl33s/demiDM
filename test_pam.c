#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <security/pam_appl.h>

static int conv_func(int num_msg, const struct pam_message **msg,
                     struct pam_response **resp, void *appdata_ptr) {
    printf("conv_func called: num_msg=%d\n", num_msg);
    
    struct pam_response *response = calloc(num_msg, sizeof(struct pam_response));
    if (!response) return PAM_BUF_ERR;
    
    for (int i = 0; i < num_msg; i++) {
        printf("  msg[%d]: style=%d, text=%s\n", i, msg[i]->msg_style, msg[i]->msg);
        response[i].resp = strdup((char *)appdata_ptr);
        response[i].resp_retcode = 0;
    }
    
    *resp = response;
    return PAM_SUCCESS;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s <username> <password>\n", argv[0]);
        return 1;
    }
    
    const char *username = argv[1];
    const char *password = argv[2];
    printf("Testing PAM auth: user=%s, password_len=%zu\n", username, strlen(password));
    
    struct pam_conv conv = {
        .conv = conv_func,
        .appdata_ptr = (void *)password,
    };
    
    pam_handle_t *pamh = NULL;
    int retval = pam_start("demidm", username, &conv, &pamh);
    printf("pam_start: %d (%s)\n", retval, pam_strerror(pamh, retval));
    if (retval != PAM_SUCCESS) {
        pam_end(pamh, retval);
        return 1;
    }
    
    retval = pam_authenticate(pamh, 0);
    printf("pam_authenticate: %d (%s)\n", retval, pam_strerror(pamh, retval));
    
    if (retval == PAM_SUCCESS) {
        retval = pam_acct_mgmt(pamh, 0);
        printf("pam_acct_mgmt: %d (%s)\n", retval, pam_strerror(pamh, retval));
    }
    
    pam_end(pamh, retval);
    return retval == PAM_SUCCESS ? 0 : 1;
}
