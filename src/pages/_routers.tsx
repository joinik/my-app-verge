import SettingsRoundedIcon from '@mui/icons-material/SettingsRounded'
import HomeRoundedIcon from '@mui/icons-material/HomeRounded'
import { createBrowserRouter, RouteObject } from 'react-router'

import HomeSvg from '@/assets/image/itemicon/home.svg?react'
import SettingsSvg from '@/assets/image/itemicon/settings.svg?react'
import HomePage from './home'
import SettingsPage from './settings'

export const navItems = [
  {
    label: 'layout.components.navigation.tabs.home',
    path: '/',
    icon: [<HomeRoundedIcon key="mui" />, <HomeSvg key="svg" />],
    Component: HomePage,
  },
  {
    label: 'layout.components.navigation.tabs.settings',
    path: '/settings',
    icon: [<SettingsRoundedIcon key="mui" />, <SettingsSvg key="svg" />],
    Component: SettingsPage,
  },
]

export const router = createBrowserRouter([
  {
    path: '/',
    Component: Layout,
    children: navItems.map(
      (item) =>
        ({
          path: item.path,
          Component: item.Component,
        } as RouteObject)
    ),
  },
])
