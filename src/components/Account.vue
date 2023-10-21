<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { VCardText } from "vuetify/components";
import { writeText } from '@tauri-apps/api/clipboard';
import { open } from '@tauri-apps/api/shell';
import { onMounted } from "vue";


interface AccountInfo {
    account_id: string
    account_name: string
    email_address: string
    alias?: string
}
interface Role {
    role_name: string
    account_id: string
    alias?: string
}
interface Credentials {
    access_key_id: string
    secret_access_key: string
    session_token: string
}
const props = defineProps<{
    account: AccountInfo
    partitionSlug: string
}>()
const roles = ref<Role[]>([]);

async function listRolesForAccount(accountId: string, partition: string) {
    const resp: Role[] = await invoke("list_roles_for", { accountId, partition });
    roles.value = resp;
}

async function openWebConsole(roleName: string, accountId: string, partition: string) {
    console.log(`opening web console for ${roleName} in ${partition} for ${accountId}`)
    await open(await invoke("open_web_console", { partition, accountId, roleName }));
}

async function copyCredentialsFor(roleName: string, accountId: string, partition: string) {
    console.log(`getting credentials from ${partition} for ${roleName}`,)
    const creds: Credentials = await invoke("get_credentials_for", { partition, accountId, roleName });
    console.log("creds", creds);
    const script = [
        `export AWS_ACCESS_KEY_ID="${creds.access_key_id}"`,
        `export AWS_SECRET_ACCESS_KEY="${creds.secret_access_key}"`,
        `export AWS_SESSION_TOKEN="${creds.session_token}"`,
    ].join("\n");
    await writeText(script);
}

onMounted(async () => {
    await listRolesForAccount(props.account.account_id, props.partitionSlug)
})
</script>

<template>
    <VCard>
        <VCardTitle>{{ $props.account.alias || $props.account.account_name }}</VCardTitle>
        <VCardSubtitle>{{ $props.account.email_address }}</VCardSubtitle>
        <VCardText v-for="role in roles">
            {{ role.alias || role.role_name }}
            <VBtn icon="mdi-open-in-new"
                @click="openWebConsole(role.role_name, $props.account.account_id, $props.partitionSlug)"></VBtn>
            <VBtn icon="mdi-console"
                @click="copyCredentialsFor(role.role_name, $props.account.account_id, $props.partitionSlug)"></VBtn>
        </VCardText>
    </VCard>
</template>