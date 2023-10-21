<script setup lang="ts">
import SessionList from "./components/SessionList.vue";
</script>

<template>
  <VApp id="inspire">
    <VAppBar>
      <VAppBarTitle>AWS Role Speed Dial</VAppBarTitle>
    </VAppBar>
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