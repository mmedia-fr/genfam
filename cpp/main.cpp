// SPDX-License-Identifier: GPL-3.0-or-later
// GenFam — point d'entrée : application Qt, moteur QML et contrôles de fabrication.
#include <QtCore/QCoreApplication>
#include <QtCore/QFileInfo>
#include <QtCore/QStringList>
#include <QtCore/QTimer>
#include <QtCore/QUrl>
#include <QtCore/QVariant>
#include <QtGui/QGuiApplication>
#include <QtGui/QIcon>
#include <QtGui/QImage>
#include <QtQml/QQmlApplicationEngine>
#include <QtQml/QQmlContext>
#include <QtQuick/QQuickWindow>
#include <QtQuickControls2/QQuickStyle>

#ifdef Q_OS_WIN
#  include <windows.h>
#  include <shellapi.h>
#endif

#include <cstdio>

/// Arguments en Unicode : sous Windows, argv est dans la page de code ANSI et un
/// fichier « Généalogie été.ged » n'arriverait pas intact (constat fait sur MMdedit).
static QStringList argumentsUnicode()
{
#ifdef Q_OS_WIN
  int nombre = 0;
  LPWSTR* natifs = CommandLineToArgvW(GetCommandLineW(), &nombre);
  if (!natifs)
    return QCoreApplication::arguments();
  QStringList liste;
  for (int i = 0; i < nombre; ++i)
    liste.append(QString::fromWCharArray(natifs[i]));
  LocalFree(natifs);
  return liste;
#else
  return QCoreApplication::arguments();
#endif
}

int main(int argc, char* argv[])
{
  QGuiApplication app(argc, argv);
  QCoreApplication::setApplicationName(QStringLiteral("GenFam"));
  QCoreApplication::setOrganizationName(QStringLiteral("M-Media"));
  QCoreApplication::setOrganizationDomain(QStringLiteral("mmedia.fr"));
  QCoreApplication::setApplicationVersion(QStringLiteral(GENFAM_VERSION));
  QGuiApplication::setWindowIcon(QIcon(QStringLiteral(":/assets/genfam.png")));

  bool smoke = false;
  QString capture;
  QString fichier;
  const QStringList arguments = argumentsUnicode();
  for (int i = 1; i < arguments.size(); ++i) {
    const QString a = arguments.at(i);
    if (a == QStringLiteral("--smoke"))
      smoke = true;
    else if (a == QStringLiteral("--capture") && i + 1 < arguments.size())
      capture = arguments.at(++i);
    else if (fichier.isEmpty())
      fichier = a;
  }

  // Fusion : seul style qui honore partout les couleurs imposées par l'interface.
  QQuickStyle::setStyle(QStringLiteral("Fusion"));

  QQmlApplicationEngine engine;
  QUrl fichierInitial;
  if (!fichier.isEmpty())
    fichierInitial = QUrl::fromLocalFile(QFileInfo(fichier).absoluteFilePath());
  engine.rootContext()->setContextProperty(QStringLiteral("fichierInitial"), fichierInitial);

  const QUrl url(QStringLiteral("qrc:/qt/qml/fr/mmedia/genfam/qml/Main.qml"));
  QObject::connect(
    &engine, &QQmlApplicationEngine::objectCreated, &app,
    [url](QObject* obj, const QUrl& objUrl) {
      if (!obj && url == objUrl)
        QCoreApplication::exit(1);
    },
    Qt::QueuedConnection);
  engine.load(url);

  if (smoke) {
    // Contrôle de fabrication : la fenêtre existe, le fichier donné est lu, et
    // chaque vue (fiche, famille, ascendants, descendants, pedigree) rend quelque chose.
    QTimer::singleShot(200, &app, [&engine, fichier] {
      if (engine.rootObjects().isEmpty()) {
        std::fprintf(stderr, "smoke: aucune fenetre creee\n");
        QCoreApplication::exit(2);
        return;
      }
      QObject* racine = engine.rootObjects().first();
      QVariant r;
      QMetaObject::invokeMethod(racine, "controle", Q_RETURN_ARG(QVariant, r));
      const QString texte = r.toString();
      std::printf("smoke: %s (%s)\n", texte.toUtf8().constData(), fichier.toUtf8().constData());
      std::fflush(stdout);
      QCoreApplication::exit(texte.startsWith(QStringLiteral("ok")) ? 0 : 3);
    });
  }

  if (!capture.isEmpty()) {
    QTimer::singleShot(800, &app, [&engine, capture] {
      auto* fenetre = engine.rootObjects().isEmpty() ? nullptr
                      : qobject_cast<QQuickWindow*>(engine.rootObjects().first());
      if (!fenetre) {
        QCoreApplication::exit(5);
        return;
      }
      QCoreApplication::exit(fenetre->grabWindow().save(capture) ? 0 : 6);
    });
  }
  return app.exec();
}
