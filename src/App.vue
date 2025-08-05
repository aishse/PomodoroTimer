<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { ref, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { PauseCircleIcon, AdjustmentsVerticalIcon, PlayCircleIcon, ForwardIcon} from "@heroicons/vue/16/solid";



const timeLeft = ref<number | null>(null); // time left in seconds
const isRunning = ref<boolean>(false); 
const isPaused = ref<boolean>(true); // state to track if the timer is paused
const state = ref<string>('Work'); // state of the pomodoro session

interface PomodoroPhase {
  name: string;
  duration: number;
}


const pomodoroPhases: PomodoroPhase[] = [
  { name: 'Work', duration: 25 },
  { name: 'ShortBreak', duration: 5 },
  { name: 'LongBreak', duration: 15}
];

const start = async () => {
  try {
   
    // if this is the first run
    if (!isRunning.value) {
      isRunning.value = true;
      isPaused.value = false;

      await invoke("start_pomodoro");
      await invoke("init_time", {
        time: pomodoroPhases.find(phase => phase.name === state.value)?.duration || 0
      });
      console.log("first start"); 
    } // if the timer is running but we wish to pause
    else {
      await invoke("toggle_pomodoro"); 
      console.log("pause!"); 
    } 

  } catch (error) {
    console.error('Error starting pomodoro:', error);
    isRunning.value = false;
  }
};

const switch_forward = async () => {
  try {
    await invoke("switch_phase");
    console.log("Phase switched!");
  } catch (error) {
    console.error('Error switching phase:', error);
  }
};
// effectively "event listeners" for the timer
onMounted(async () => {
  const initialDuration = pomodoroPhases.find(phase => phase.name === state.value)?.duration;
  timeLeft.value = initialDuration || null;

  await invoke("init_time", { time: initialDuration || 0 });

  listen<string>('phase_changed', async (event) => {
    state.value = event.payload;
   
    const duration = pomodoroPhases.find(phase => phase.name === state.value)?.duration;
    timeLeft.value = duration || null;
    
    if (duration) {
      await invoke('init_time', { time: duration });
    }
  });

  // reset the values, move the pomodoro state forward
  listen('done', () => {
    console.log('Timer completed - phase auto-switched!');``
    isRunning.value = false;
    isPaused.value = true;
  

  });

  listen<boolean>('toggle', event => {
    isPaused.value = event.payload;
    console.log('Timer paused');
  });

  // Listen for manual timer stops
  listen('timer_stopped', () => {
    console.log('Timer was manually stopped');
    isRunning.value = false;
    isPaused.value = true;
   
  });

  listen<number>('tick', event => {
    timeLeft.value = event.payload;
    console.log('Tick received:', event.payload);
  });

});

</script>

<template>
  <main class=" bg-blue-300 container mx-auto px-4 py-8 flex flex-col min-w-screen items-center justify-center min-h-screen">
    <h1 class="text-4xl font-bold text-gray-800 mb-6">Lock-in-inator</h1>
    <div class="flex flex-col items-center justify-center w-full">
      <h2 class="text-2xl text-gray-600 mx-2 text-center">{{  `00:${timeLeft}` }}</h2>

    </div>
    
    <h1 v-if="isRunning && !isPaused" class="text-xl text-green-600 mb-6">Running...</h1>
    <h1 v-else class="text-xl text-gray-500 mb-6">Ready to start your Pomodoro session</h1>
    <h1 class="text-xl text-gray-500 mb-6">Current State: {{ state }}</h1>

    <div class="flex flex-row items-center justify-center space-x-4">
       <button 
      @click="" 
      class="flex items-center justify-center transition-all hover:opacity-60 duration-[500ms] ease-in-out active:scale-60"
      >
        <AdjustmentsVerticalIcon class="h-8 w-8" />
      </button>
      <button 
      @click="start" 
      class="flex items-center justify-center transition-all hover:opacity-85 duration-[500ms] ease-in-out active:scale-60"
      >
      <PlayCircleIcon v-if="isPaused" class="h-12 w-12 text-gray-800 transition-opacity duration-[500ms]" />
      <PauseCircleIcon v-else class="h-12 w-12 text-gray-800 transition-opacity duration-[500ms]" />

      </button>
      <button 
      @click="switch_forward" 
      class="flex items-center justify-center transition-all hover:opacity-60 duration-[500ms] ease-in-out active:scale-60"
      >
      <ForwardIcon class="h-8 w-8" />
      </button>
    </div>
    </main>
</template>

