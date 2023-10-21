<script setup lang="ts">
import { onBeforeUnmount, ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/api/shell';
import AccountList from "./AccountList.vue";
import CountDown from "./CountDown.vue";

const props = defineProps<{
  partition: Partition
}>()

interface Partition {
  start_url: string
  account_id: string
  region: string
  slug: string
  status?: string
  message?: string
}
interface Confirmation {
  partition: string
  user_code: string
  device_code: string
  confirmation_url: string
  polling_interval: number
  expires_at: number
}

const sessionState = ref("");
const session = ref<Confirmation>();
const checkToken = ref();
const expiresAt = ref<Date | undefined>();

onBeforeUnmount(() => {
  clearInterval(checkToken.value);
  checkToken.value = null;
});

await listen('session_state', (event: any) => {
  console.log("session_state event received")
  sessionState.value = event.payload;
})

await listen('authorize_device', async (event: any) => {
  console.log("authorize_device event received", event.payload)
})

async function tryAuth(partition: string) {
  console.log('starting on', partition)
  const payload = await invoke("authorize_device", { authEvent: { partition_name: partition } });
  if ((payload as any).type == "Success") {
    // @ts-ignore-next-line
    console.log(`Expires in ${Math.round((payload!.expires_at - Date.now()) / 60000)} mins`, payload)
    // @ts-ignore-next-line
    expiresAt.value = new Date(payload!.expires_at);
  } else if ((payload as any).type == "NeedsConfirmation") {
    console.log("NeedsConfirmation received", payload)
    session.value = payload as Confirmation;
    await open(session.value.confirmation_url);
    checkToken.value = setInterval(async () => {
      const confirmation: Confirmation = (payload as Confirmation)
      const checkResult = await invoke("check_device_token", {
        tokenEvent: {
          device_code: confirmation.device_code,
          user_code: session?.value?.user_code,
          partition: confirmation.partition,
        }
      });
      console.log("checking token", confirmation, checkResult)
      if (checkResult === "Done") {
        console.log("Done, ending timer")
        clearInterval(checkToken.value);
        checkToken.value = null;
      } else if (checkResult === "Pending") {
        console.log("still pending")
      } else {
        console.log("other result", checkResult)
        clearInterval(checkToken.value);
        checkToken.value = null;
      }
    }, session.value.polling_interval * 1000)
  } else {
    console.log("other auth event received", payload)
  }
}
tryAuth(props.partition.slug);
</script>

<template>
  <VRow v-if="expiresAt === undefined || expiresAt < new Date(Date.now())">
    <VCol cols=2>
      <VBtn @click="tryAuth($props.partition.slug)">Try auth</VBtn>
    </VCol>
    <VCol cols=8>
      <p>For {{ $props.partition.slug }} you should expect to see
        <code>{{ session?.user_code || "pending" }}</code> when you go
        <a target="_blank" :href="session?.confirmation_url">here</a>.
      </p>
    </VCol>
  </VRow>
  <VRow v-if="expiresAt !== undefined && expiresAt > new Date(Date.now())">
    <VCol cols=8>
      <p>The {{ $props.partition.slug }} session is ready and expires in:
        <CountDown :countTo=expiresAt />
      </p>
    </VCol>
    <VCol cols=4>
      <VBtn @click="tryAuth($props.partition.slug)">Refresh Credentials</VBtn>
    </VCol>
  </VRow>
  <AccountList :partitionSlug=$props.partition.slug />
</template>