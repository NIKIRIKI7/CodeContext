from abc import ABC, abstractmethod
from typing import List


class ImportResolutionStrategy(ABC):
    """Базовый интерфейс для стратегий разрешения импортов"""

    @abstractmethod
    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        """
        Проверяет, соответствует ли файл запрошенному импорту.
        :param p_no_ext: Нормализованный путь файла в проекте (без расширения)
        :param core_imp: Очищенная от алиасов строка импорта
        :param suffix: Строка импорта с ведущим слэшем (например, '/entities/user')
        :param parts: Строка импорта, разбитая по слэшам
        """
        pass


class StandardImportStrategy(ImportResolutionStrategy):
    """Стратегия для стандартных прямых импортов (например, import { x } from 'entities/user.ts')"""

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        return p_no_ext.endswith(suffix) or p_no_ext == core_imp


class FSDImportStrategy(ImportResolutionStrategy):
    """
    Стратегия для Feature-Sliced Design (FSD).
    Ищет Barrel-файлы вида index.ts или ui/index.vue внутри папки импорта.
    """

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        return p_no_ext.endswith(f"{suffix}/index") or p_no_ext.endswith(f"{suffix}/ui/index")


class AtomicDesignImportStrategy(ImportResolutionStrategy):
    """
    Стратегия для Atomic Design / React паттернов.
    Обрабатывает компоненты, где папка и файл называются одинаково (например, Button/Button.tsx).
    """

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        if not parts:
            return False
        component_name = parts[-1]
        return p_no_ext.endswith(f"{suffix}/{component_name}")


class DDDImportStrategy(ImportResolutionStrategy):
    """
    Стратегия для Domain-Driven Design (DDD) / Python пакетов.
    Ищет точки входа в модули (__init__.py, main.py).
    """

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        return p_no_ext.endswith(f"{suffix}/__init__") or p_no_ext.endswith(f"{suffix}/main")


class MonorepoImportStrategy(ImportResolutionStrategy):
    """
    Стратегия для монорепозиториев (Lerna, NX, Turborepo, pnpm workspaces).

    Обрабатывает импорты вида:
      - import { Button } from '@myorg/shared-ui'        → packages/shared-ui/src/index.ts
      - import { utils } from '@myorg/core/utils/format'  → packages/core/src/utils/format.ts
      - import { X } from 'shared-ui'                     → packages/shared-ui/src/index.ts
      - import { X } from 'shared-ui/button'              → packages/shared-ui/src/button.ts

    Ищет совпадения с пакетами, расположенными в типовых
    директориях монорепозитория: packages/, libs/, apps/, modules/.
    """

    MONOREPO_DIRS = ("/packages/", "/libs/", "/apps/", "/modules/")

    def is_match(self, p_no_ext: str, core_imp: str, suffix: str, parts: List[str]) -> bool:
        if not parts:
            return False

        p_norm = p_no_ext.replace("\\", "/")

        for mono_dir in self.MONOREPO_DIRS:
            idx = p_norm.find(mono_dir)
            if idx == -1:
                continue

            after_dir = p_norm[idx + len(mono_dir):]
            path_segments = after_dir.split("/")
            if not path_segments:
                continue

            pkg_in_path = path_segments[0]

            if pkg_in_path not in parts:
                continue

            pkg_idx = parts.index(pkg_in_path)
            path_after_pkg = "/".join(path_segments[1:])
            parts_after_pkg = parts[pkg_idx + 1:]

            if not parts_after_pkg:
                if path_after_pkg in ("src/index", "lib/index", "index"):
                    return True
            else:
                suffix_from_parts = "/".join(parts_after_pkg)
                if path_after_pkg in (suffix_from_parts,
                                      f"src/{suffix_from_parts}",
                                      f"lib/{suffix_from_parts}"):
                    return True

        return False