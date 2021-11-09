<template>
<Hud />
<Dialogue v-if="event && event.dialogue" :dialogue="event.dialogue" :effects="event.effects" @done="nextEvent" @select="selectChoice" />
<div class="planning">
  <header>
    <div :class="{active: page == PAGES.PLAN}" @click="page = PAGES.PLAN">Plan</div>
    <div :class="{active: page == PAGES.COALITION}" @click="page = PAGES.COALITION">Coalition</div>
    <div :class="{active: page == PAGES.DASHBOARD}" @click="page = PAGES.DASHBOARD">Dashboard</div>
  </header>

  <Plan v-if="page == PAGES.PLAN" />
  <Coalition v-else-if="page == PAGES.COALITION" />
  <Dashboard v-else-if="page == PAGES.DASHBOARD" />
</div>
</template>

<script>
import game from '/src/game';
import state from '/src/state';
import display from 'lib/display';
import Hud from 'components/Hud.vue';
import Coalition from './Coalition.vue';
import Dashboard from './Dashboard.vue';
import Plan from './Plan.vue';
import EventsMixin from 'components/EventsMixin';
import EVENTS from '/assets/content/events.json';

const PAGES = {
  PLAN: 0,
  COALITION: 1,
  DASHBOARD: 2,
};

export default {
  mixins: [EventsMixin],
  components: {
    Hud,
    Coalition,
    Dashboard,
    Plan,
  },
  created() {
    this.PAGES = PAGES;
  },
  mounted() {
    this.showEvent();
  },
  activated() {
    this.showEvent();
  },
  data() {
    let events = game.roll.planningEvents();

    // Group events by pages
    let eventsByPage = Object.keys(PAGES).reduce((acc, k) => {
      acc[k] = [];
      return acc;
    }, {});
    eventsByPage[null] = [];
    events.forEach(([ev_id, region_id]) => {
      let ev = EVENTS[ev_id];
      let page = null;
      let parts = ev.name.split(':');
      if (parts.length > 1) {
        page = parts.shift();
      }
      eventsByPage[page].push([ev_id, region_id]);
    });

    return {
      state,
      events: eventsByPage[null],
      eventsByPage,
      page: PAGES.PLAN,
    }
  },
  computed: {
    demand() {
      return display.outputs(state.gameState.output_demand);
    },
    emissions() {
      return display.gtco2eq(state.gameState.byproducts);
    }
  },
  methods: {
    select(p) {
      if (PAGES[p] == PAGES.CONTINUE) {
        state.phase = 'EVENTS';
      } else {
        this.page = PAGES[p];
        this.events = this.eventsByPage[p];
        this.showEvent();
      }
    },
  }
}
</script>

<style>
.planning {
  background: #ffecc7;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}
.planning--menu {
  padding: 1em;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
}
.planning--menu button {
  width: 96px;
  height: 96px;
  margin: 0 10% 1em;
  padding: 0.25em 0.5em;
  border-width: 4px;
  justify-self: center;
}
.planning--menu img {
  max-width: 36px;
}

.pip {
  width: 22px;
  vertical-align: middle;
}
.pips {
  padding: 0.5em;
  margin: 0.25em;
  position: relative;
  text-align: center;
  font-size: 1.2em;
  color: #fff;
}
.pips--buy {
  cursor: pointer;
  user-select: none;
  border-radius: 0.2em;
  background: rgba(0,0,0,0.1);
}
.pips--buy:hover {
  background: rgba(255,255,255,0.3);
}
.pip-in-use {
  opacity: 0.5;
}

.planning--page {
  display: flex;
  flex-direction: column;
  flex: 1;
}
.planning--page .cards {
  flex: 1;
  margin-top: 1em;
}
.planning--page .card header img {
  width: 12px;
  vertical-align: middle;
  margin-top: -2px;
}

.project--upgrade--title {
  display: flex;
  font-size: 0.75em;
  justify-content: space-between;
  border-bottom: 1px dashed;
  padding: 0 0 0.25em 0;
  margin-bottom: 0.5em;
}
.project--upgrade--title button {
  padding: 0 0.5em;
}
.project--upgrade .effects {
    font-size: 0.8em;
    padding: 0.1em 0.3em;
    border: none;
    background: rgba(0,0,0,0.1);
}
.project--upgrade img,
.project--upgrade .effects img {
  width: 16px;
  height: 16px;
  vertical-align: middle;
}

.planning--demand {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-evenly;
  padding: 1em;
  font-size: 1.1em;
}

.planning > header {
  display: flex;
  border-bottom: 1px solid #000;
}
.planning > header div {
  flex: 1;
  text-align: center;
  padding: 0.25em;
  border-right: 1px solid #000;
}
.planning > header div.active {
  background: #e47d4a;
  color: #fff;
}
.planning > header div:last-child {
  border-right: none;
}

.planning--page > footer {
  display: flex;
  justify-content: space-between;
}
</style>