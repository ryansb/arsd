<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { open } from "@tauri-apps/plugin-shell";
import { ref } from "vue";
import { onMounted } from "vue";
import type { AccountInfo, Credentials, Role } from "../store";

const props = defineProps<{
  account: AccountInfo;
  partitionSlug: string;
}>();
const roles = ref<Role[]>([]);
const snackbar = ref(false);
const snackbarText = ref("");

async function listRolesForAccount(accountId: string, partition: string) {
  const resp: Role[] = await invoke("list_roles_for", {
    accountId,
    partition,
  });
  roles.value = resp.sort(roleCmp);
}

async function openWebConsole(
  roleName: string,
  accountId: string,
  partition: string,
) {
  await open(
    await invoke("open_web_console", { partition, accountId, roleName }),
  );
}

async function copyConsoleLink(
  roleName: string,
  accountId: string,
  partition: string,
) {
  await writeText(
    await invoke("open_web_console", { partition, accountId, roleName }),
  );
  snackbarText.value = "Copied console link to clipboard";
  snackbar.value = true;
}

const snackMessages = {
  CREDS: "Copied credentials to clipboard",
  CONSOLE_LINK: "Copied console link to clipboard",
  EMAIL: "Copied account email to clipboard",
  ID: "Copied account ID to clipboard",
};

async function copyCredentialsFor(
  roleName: string,
  accountId: string,
  partition: string,
) {
  const creds: Credentials = await invoke("get_credentials_for", {
    partition,
    accountId,
    roleName,
  });
  const script = [
    `export AWS_ACCESS_KEY_ID="${creds.access_key_id}"`,
    `export AWS_SECRET_ACCESS_KEY="${creds.secret_access_key}"`,
    `export AWS_SESSION_TOKEN="${creds.session_token}"`,
  ].join("\n");
  await writeText(script);
  snackbarText.value = snackMessages.CREDS;
  snackbar.value = true;
}

onMounted(async () => {
  await listRolesForAccount(props.account.account_id, props.partitionSlug);
});

function roleCmp(a: Role, b: Role): number {
  if ((a.alias || a.role_name) > (b.alias || b.role_name)) {
    return 1;
  } else if ((a.alias || a.role_name) < (b.alias || b.role_name)) {
    return -1;
  }
  return 0;
}
</script>

<template>
    <VCard height="100%" min-width="300px">
        <VCardTitle>{{
            $props.account.alias || $props.account.account_name
        }}</VCardTitle>
        <VCardSubtitle
            @click="
                snackbarText = snackMessages.EMAIL;
                snackbar = true;
                writeText($props.account.email_address);
            "
            >{{ $props.account.email_address }}
        </VCardSubtitle>
        <VCardSubtitle
            @click="
                snackbarText = snackMessages.ID;
                snackbar = true;
                writeText($props.account.account_id);
            "
            >{{ $props.account.account_id }}
        </VCardSubtitle>
        <VSnackbar v-model="snackbar" :timeout="2000" top>
            <VBtn icon="mdi-close" variant="text" @click="snackbar = false" />
            <span>{{ snackbarText }}</span>
        </VSnackbar>
        <VTable class="mx-4 my-2">
            <tr v-for="role in roles" :key="role.role_name">
                <td>{{ role.alias || role.role_name }}</td>
                <td>
                    <VBtnGroup class="float-right py-2" rounded="1" divided>
                        <VBtn
                            color="secondary"
                            @click="
                                copyCredentialsFor(
                                    role.role_name,
                                    $props.account.account_id,
                                    $props.partitionSlug,
                                )
                            "
                        >
                            <VIcon>mdi-console</VIcon>
                            <VTooltip activator="parent" open-delay="300"
                                >Copy credentials to clipboard
                            </VTooltip>
                        </VBtn>
                        <VBtn
                            color="secondary"
                            @click="
                                copyConsoleLink(
                                    role.role_name,
                                    $props.account.account_id,
                                    $props.partitionSlug,
                                )
                            "
                        >
                            <VIcon>mdi-link-plus</VIcon>
                            <VTooltip activator="parent" open-delay="300"
                                >Copy console link to clipboard
                            </VTooltip>
                        </VBtn>
                        <VBtn
                            color="primary"
                            @click="
                                openWebConsole(
                                    role.role_name,
                                    $props.account.account_id,
                                    $props.partitionSlug,
                                )
                            "
                        >
                            <VIcon>mdi-open-in-new</VIcon>
                            <VTooltip activator="parent" open-delay="300"
                                >Open in web console</VTooltip
                            >
                        </VBtn>
                    </VBtnGroup>
                </td>
            </tr>
        </VTable>
    </VCard>
</template>
