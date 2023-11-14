<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { VCardText } from "vuetify/components";
import { writeText } from '@tauri-apps/api/clipboard';
import { open } from '@tauri-apps/api/shell';
import { onMounted } from "vue";
import { AccountInfo, Role, Credentials } from "../store";


const props = defineProps<{
    account: AccountInfo
    partitionSlug: string
}>()
const roles = ref<Role[]>([]);

async function listRolesForAccount(accountId: string, partition: string) {
    const resp: Role[] = await invoke("list_roles_for", { accountId, partition });
    roles.value = resp.sort(roleCmp);
}

async function openWebConsole(roleName: string, accountId: string, partition: string) {
    await open(await invoke("open_web_console", { partition, accountId, roleName }));
}

async function copyCredentialsFor(roleName: string, accountId: string, partition: string) {
    const creds: Credentials = await invoke("get_credentials_for", { partition, accountId, roleName });
    const script = [
        `export AWS_ACCESS_KEY_ID="${creds.access_key_id}"`,
        `export AWS_SECRET_ACCESS_KEY="${creds.secret_access_key}"`,
        `export AWS_SESSION_TOKEN="${creds.session_token}"`,
    ].join("\n");
    await writeText(script);
    snackbar.value = true;
}

onMounted(async () => {
    await listRolesForAccount(props.account.account_id, props.partitionSlug)
})

function roleCmp(a: Role, b: Role): number {
    if ((a.alias || a.role_name) > (b.alias || b.role_name)) {
        return 1
    } else if ((a.alias || a.role_name) < (b.alias || b.role_name)) {
        return -1
    }
    return 0
}


let snackbar = ref(false);
</script>

<template>
    <VCard>
        <VCardTitle>{{ $props.account.alias || $props.account.account_name }}</VCardTitle>
        <VCardSubtitle @click="snackbar = true; writeText($props.account.email_address)">{{ $props.account.email_address }}
        </VCardSubtitle>
        <VCardSubtitle @click="snackbar = true; writeText($props.account.account_id)">{{ $props.account.account_id }}
        </VCardSubtitle>
        <VSnackbar v-model="snackbar" :timeout="2000" top>
            <VBtn icon="mdi-close" variant="text" @click="snackbar = false" />
            <span>Copied to clipboard</span>
        </VSnackbar>
        <VCardText v-for="role in roles" :key="role.role_name">
            <span>{{ role.alias || role.role_name }}</span>
            <VBtn color="primary" rounded="pill" class="float-right mb-2 ml-3"
                @click="openWebConsole(role.role_name, $props.account.account_id, $props.partitionSlug)">
                <VIcon>mdi-open-in-new</VIcon>
                <VTooltip activator="parent" location="top" open-delay="500">Open in web console</VTooltip>
            </VBtn>
            <VBtn color="secondary" rounded="pill" class="float-right mb-2 ml-3"
                @click="copyCredentialsFor(role.role_name, $props.account.account_id, $props.partitionSlug)">
                <VIcon>mdi-console</VIcon>
                <VTooltip activator="parent" location="top" open-delay="300">Copy credentials to clipboard</VTooltip>
            </VBtn>
        </VCardText>
    </VCard>
</template>