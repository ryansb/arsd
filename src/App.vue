<script setup lang="ts">
import { ref, watch } from "vue";
import SessionList from "./components/SessionList.vue";
import SortSelect from "./components/SortSelect.vue";
import SearchBar from "./components/SearchBar.vue";
import { nextTick } from "vue";

const drawer = ref(false);
const search = ref(false);
const searchBox = ref<HTMLInputElement | null>(null);

watch(search, (isShown) => {
  if (isShown) {
    nextTick(() => {
      searchBox.value && searchBox.value.focus();
    })
  }
});

</script>

<template>
  <VApp>
    <VAppBar scroll-behavior="elevate" color="surface">
      <VAppBarNavIcon @click="drawer = !drawer">
        <VIcon>mdi-menu</VIcon>
      </VAppBarNavIcon>
      <SearchBar />
      <VSpacer />
      <SortSelect />
      <VAppBarNavIcon>
        <VIcon>mdi-refresh</VIcon>
      </VAppBarNavIcon>
    </VAppBar>
    <VNavigationDrawer v-model="drawer">
      <VList>
        <VListItem>
          <VListItemTitle>Services</VListItemTitle>
        </VListItem>
      </VList>
    </VNavigationDrawer>
    <VMain>
      <VContainer>
        <Suspense>
          <SessionList />
        </Suspense>
        <VRow>
          <VCol>
            <h1>Wish list</h1>
            <VList>
              <ul>
                <li>Support default heirarchy of roles (prefer readonly, then working, then ...)</li>
                <li>Show a top 3 frecency-based list first up</li>
                <li>Allow tagging for environments or other account groupings</li>
                <li>Add <a
                    href="https://docs.aws.amazon.com/sdkref/latest/guide/feature-sso-credentials.html#sso-token-config">sso-session</a>
                  section to aws config</li>
                <li>Dump available roles to AWS config</li>
              </ul>
            </VList>
          </VCol>
        </VRow>
      </VContainer>
    </VMain>
  </VApp>
</template>